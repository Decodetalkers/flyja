use smithay::{
    backend::input::TabletToolDescriptor,
    delegate_commit_timing, delegate_data_device, delegate_ext_data_control,
    delegate_input_method_manager, delegate_keyboard_shortcuts_inhibit, delegate_output,
    delegate_pointer_constraints, delegate_pointer_gestures, delegate_primary_selection,
    delegate_relative_pointer, delegate_seat, delegate_shm, delegate_tablet_manager,
    delegate_text_input_manager, delegate_viewporter, delegate_virtual_keyboard_manager,
    delegate_xdg_activation, delegate_xdg_decoration, delegate_xdg_foreign,
    desktop::{PopupKind, PopupManager, Space, WindowSurfaceType},
    input::{
        Seat, SeatHandler, SeatState,
        keyboard::XkbConfig,
        pointer::{CursorImageStatus, PointerHandle},
    },
    reexports::{
        calloop::{
            EventLoop, Interest, LoopHandle, LoopSignal, Mode, PostAction, generic::Generic,
        },
        wayland_protocols::xdg::decoration::{
            self as xdg_decoration,
            zv1::server::zxdg_toplevel_decoration_v1::Mode as DecorationMode,
        },
        wayland_server::{
            Display, DisplayHandle, Resource,
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::{wl_buffer::WlBuffer, wl_seat::WlSeat, wl_surface::WlSurface},
        },
    },
    utils::{Clock, Logical, Monotonic, Point, Rectangle},
    wayland::{
        buffer::BufferHandler,
        commit_timing::{CommitTimerState, CommitTimingManagerState},
        compositor::{CompositorClientState, CompositorState},
        input_method::{
            InputMethodHandler, InputMethodManagerState, PopupSurface as ImPopupSurface,
        },
        keyboard_shortcuts_inhibit::{
            KeyboardShortcutsInhibitHandler, KeyboardShortcutsInhibitState,
            KeyboardShortcutsInhibitor,
        },
        output::{OutputHandler, OutputManagerState},
        pointer_constraints::{
            PointerConstraintsHandler, PointerConstraintsState, with_pointer_constraint,
        },
        pointer_gestures::PointerGesturesState,
        relative_pointer::RelativePointerManagerState,
        seat::WaylandFocus,
        selection::{
            SelectionHandler, SelectionSource, SelectionTarget,
            data_device::{
                DataDeviceHandler, DataDeviceState, WaylandDndGrabHandler, set_data_device_focus,
            },
            ext_data_control::{DataControlHandler, DataControlState},
            primary_selection::{
                PrimarySelectionHandler, PrimarySelectionState, set_primary_focus,
            },
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            decoration::{XdgDecorationHandler, XdgDecorationState},
        },
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
        tablet_manager::{TabletManagerState, TabletSeatHandler},
        text_input::TextInputManagerState,
        viewporter::ViewporterState,
        virtual_keyboard::VirtualKeyboardManagerState,
        xdg_activation::{
            XdgActivationHandler, XdgActivationState, XdgActivationToken, XdgActivationTokenData,
        },
        xdg_foreign::{XdgForeignHandler, XdgForeignState},
    },
};
use flyja_logic::TopElementMap;
use std::sync::Arc;

use crate::shell::element::WindowElement;

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}
impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        println!("initialized");
    }

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        println!("disconnected");
    }
}

pub trait Backend {
    const HAS_RELATIVE_MOTION: bool = false;
    const HAS_GUSTURES: bool = false;
    fn seat_name(&self) -> String;
}

pub struct FlyjaState<BackendData: Backend + 'static> {
    pub backend_data: BackendData,
    pub start_time: std::time::Instant,

    pub socket_name: Option<String>,
    pub display_handle: DisplayHandle,
    pub handle: LoopHandle<'static, Self>,
    pub signal: LoopSignal,

    // desktop
    pub space: Space<WindowElement>,
    pub map: TopElementMap,
    pub popups: PopupManager,

    // smithay state
    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub output_manager_state: OutputManagerState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<FlyjaState<BackendData>>,
    pub primary_selection_state: PrimarySelectionState,
    pub data_control_state: DataControlState,
    pub viewporter_state: ViewporterState,
    pub xdg_shell_state: XdgShellState,
    pub xdg_foreign_state: XdgForeignState,
    pub xdg_decoration_state: XdgDecorationState,
    pub xdg_activation_state: XdgActivationState,
    pub commit_timing_manager_state: CommitTimingManagerState,
    pub keyboard_shortcuts_inhibit_state: KeyboardShortcutsInhibitState,

    pub cursor_status: CursorImageStatus,
    pub seat_name: String,
    pub seat: Seat<Self>,
    pub pointer: PointerHandle<Self>,
    pub clock: Clock<Monotonic>,
}
impl<BackendData: Backend + 'static> FlyjaState<BackendData> {
    pub fn init(
        display: Display<Self>,
        event_loop: &EventLoop<'static, Self>,
        backend_data: BackendData,
        listen_on_socket: bool,
    ) -> Self {
        let handle = event_loop.handle();
        let dh = display.handle();
        let clock: Clock<Monotonic> = Clock::new();

        let socket_name = if listen_on_socket {
            let source = ListeningSocketSource::new_auto().unwrap();
            let socket_name = source.socket_name().to_string_lossy().into_owned();
            handle
                .insert_source(source, |client_stream, _, data| {
                    if let Err(err) = data
                        .display_handle
                        .insert_client(client_stream, Arc::new(ClientState::default()))
                    {
                        tracing::warn!("Error adding wayland client: {err}");
                    }
                })
                .expect("Failed to init wayland socket source");
            tracing::info!(name = socket_name, "Listening on wayland socket");
            Some(socket_name)
        } else {
            None
        };
        handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, data| {
                    profiling::scope!("dispatch_clients");
                    unsafe {
                        display.get_mut().dispatch_clients(data).unwrap();
                    }
                    Ok(PostAction::Continue)
                },
            )
            .expect("Failed to init wayland server source");

        // init globals
        let compositor_state = CompositorState::new::<Self>(&dh);
        let data_device_state = DataDeviceState::new::<Self>(&dh);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);

        let primary_selection_state = PrimarySelectionState::new::<Self>(&dh);
        let data_control_state =
            DataControlState::new::<Self, _>(&dh, Some(&primary_selection_state), |_| true);
        let mut seat_state = SeatState::<Self>::new();
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let viewporter_state = ViewporterState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&dh);
        let xdg_activation_state = XdgActivationState::new::<Self>(&dh);

        let xdg_foreign_state = XdgForeignState::new::<Self>(&dh);
        let commit_timing_manager_state = CommitTimingManagerState::new::<Self>(&dh);

        // input and etc
        TextInputManagerState::new::<Self>(&dh);
        InputMethodManagerState::new::<Self, _>(&dh, |_client| true);
        VirtualKeyboardManagerState::new::<Self, _>(&dh, |_client| true);
        if BackendData::HAS_RELATIVE_MOTION {
            RelativePointerManagerState::new::<Self>(&dh);
        }
        PointerConstraintsState::new::<Self>(&dh);
        if BackendData::HAS_GUSTURES {
            PointerGesturesState::new::<Self>(&dh);
        }

        TabletManagerState::new::<Self>(&dh);

        // init input
        let seat_name = backend_data.seat_name();
        let mut seat = seat_state.new_wl_seat(&dh, &seat_name);

        let pointer = seat.add_pointer();
        seat.add_keyboard(XkbConfig::default(), 200, 2)
            .expect("We need keyboard");
        let keyboard_shortcuts_inhibit_state = KeyboardShortcutsInhibitState::new::<Self>(&dh);
        let signal = event_loop.get_signal();
        let start_time = std::time::Instant::now();

        Self {
            start_time,
            backend_data,
            display_handle: dh,
            socket_name,
            handle,
            signal,
            space: Space::default(),
            map: TopElementMap::new(flyja_logic::SizeAndPos::default()),
            popups: PopupManager::default(),

            compositor_state,
            data_device_state,
            output_manager_state,
            primary_selection_state,
            xdg_activation_state,
            xdg_decoration_state,
            xdg_shell_state,
            xdg_foreign_state,
            keyboard_shortcuts_inhibit_state,
            seat_name,
            seat_state,
            seat,
            shm_state,
            commit_timing_manager_state,
            data_control_state,

            pointer,
            clock,
            viewporter_state,
            cursor_status: CursorImageStatus::default_named(),
        }
    }
    pub fn surface_under(
        &self,
        pos: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        self.space
            .element_under(pos)
            .and_then(|(window, location)| {
                window
                    .surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)
                    .map(|(s, p)| (s, (p + location).to_f64()))
            })
    }
}

impl<BackendData: Backend> ShmHandler for FlyjaState<BackendData> {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}
delegate_shm!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> DataDeviceHandler for FlyjaState<BackendData> {
    fn data_device_state(&mut self) -> &mut DataDeviceState {
        &mut self.data_device_state
    }
}
delegate_data_device!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> PrimarySelectionHandler for FlyjaState<BackendData> {
    fn primary_selection_state(&mut self) -> &mut PrimarySelectionState {
        &mut self.primary_selection_state
    }
}

delegate_primary_selection!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> DataControlHandler for FlyjaState<BackendData> {
    fn data_control_state(&mut self) -> &mut DataControlState {
        &mut self.data_control_state
    }
}

delegate_ext_data_control!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> WaylandDndGrabHandler for FlyjaState<BackendData> {}

impl<BackendData: Backend> SeatHandler for FlyjaState<BackendData> {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;
    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        self.cursor_status = image
    }

    // TODO: adjust later
    fn focus_changed(&mut self, seat: &Seat<Self>, target: Option<&Self::KeyboardFocus>) {
        let dh = &self.display_handle;

        let wl_surface = target.and_then(WaylandFocus::wl_surface);

        let focus = wl_surface.and_then(|s| dh.get_client(s.id()).ok());
        set_data_device_focus(dh, seat, focus.clone());
        set_primary_focus(dh, seat, focus);
    }
}

delegate_seat!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> SelectionHandler for FlyjaState<BackendData> {
    type SelectionUserData = ();
}

impl<BackendData: Backend + 'static> FlyjaState<BackendData> {}

impl<BackendData: Backend> BufferHandler for FlyjaState<BackendData> {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl<BackendData: Backend> OutputHandler for FlyjaState<BackendData> {}

delegate_output!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> XdgForeignHandler for FlyjaState<BackendData> {
    fn xdg_foreign_state(&mut self) -> &mut XdgForeignState {
        &mut self.xdg_foreign_state
    }
}

impl<BackendData: Backend> XdgDecorationHandler for FlyjaState<BackendData> {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
        // Set the default to client side
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(Mode::ClientSide);
        });
    }
    fn request_mode(&mut self, toplevel: ToplevelSurface, mode: DecorationMode) {
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;

        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(match mode {
                DecorationMode::ServerSide => Mode::ServerSide,
                _ => Mode::ClientSide,
            });
        });

        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
        }
    }
    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(Mode::ClientSide);
        });

        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
        }
    }
}
delegate_xdg_decoration!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> XdgActivationHandler for FlyjaState<BackendData> {
    fn activation_state(&mut self) -> &mut XdgActivationState {
        &mut self.xdg_activation_state
    }

    fn token_created(&mut self, _token: XdgActivationToken, data: XdgActivationTokenData) -> bool {
        if let Some((serial, seat)) = data.serial {
            let keyboard = self.seat.get_keyboard().unwrap();
            Seat::from_resource(&seat) == Some(self.seat.clone())
                && keyboard
                    .last_enter()
                    .map(|last_enter| serial.is_no_older_than(&last_enter))
                    .unwrap_or(false)
        } else {
            false
        }
    }

    fn request_activation(
        &mut self,
        _token: XdgActivationToken,
        token_data: XdgActivationTokenData,
        surface: WlSurface,
    ) {
        if token_data.timestamp.elapsed().as_secs() < 10 {
            // Just grant the wish
            let w = self
                .space
                .elements()
                .find(|window| window.wl_surface().map(|s| *s == surface).unwrap_or(false))
                .cloned();
            if let Some(window) = w {
                self.space.raise_element(&window, true);
            }
        }
    }
}
delegate_xdg_activation!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

delegate_xdg_foreign!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

delegate_viewporter!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

delegate_commit_timing!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

delegate_text_input_manager!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> InputMethodHandler for FlyjaState<BackendData> {
    fn new_popup(&mut self, surface: ImPopupSurface) {
        if let Err(err) = self.popups.track_popup(PopupKind::from(surface)) {
            tracing::warn!("Failed to track popup: {}", err);
        }
    }

    fn popup_repositioned(&mut self, _: ImPopupSurface) {}

    fn dismiss_popup(&mut self, surface: ImPopupSurface) {
        if let Some(parent) = surface.get_parent().map(|parent| parent.surface.clone()) {
            let _ = PopupManager::dismiss_popup(&parent, &PopupKind::from(surface));
        }
    }

    fn parent_geometry(&self, parent: &WlSurface) -> Rectangle<i32, smithay::utils::Logical> {
        self.space
            .elements()
            .find_map(|window| {
                (window.wl_surface().as_deref() == Some(parent)).then(|| window.geometry())
            })
            .unwrap_or_default()
    }
}

delegate_input_method_manager!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> KeyboardShortcutsInhibitHandler for FlyjaState<BackendData> {
    fn keyboard_shortcuts_inhibit_state(&mut self) -> &mut KeyboardShortcutsInhibitState {
        &mut self.keyboard_shortcuts_inhibit_state
    }

    fn new_inhibitor(&mut self, inhibitor: KeyboardShortcutsInhibitor) {
        // Just grant the wish for everyone
        inhibitor.activate();
    }
}

delegate_keyboard_shortcuts_inhibit!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

delegate_virtual_keyboard_manager!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

delegate_pointer_gestures!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

delegate_relative_pointer!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> PointerConstraintsHandler for FlyjaState<BackendData> {
    fn new_constraint(&mut self, surface: &WlSurface, pointer: &PointerHandle<Self>) {
        // XXX region
        let Some(current_focus) = pointer.current_focus() else {
            return;
        };
        if current_focus.wl_surface().as_deref() == Some(surface) {
            with_pointer_constraint(surface, pointer, |constraint| {
                constraint.unwrap().activate();
            });
        }
    }

    fn cursor_position_hint(
        &mut self,
        surface: &WlSurface,
        pointer: &PointerHandle<Self>,
        location: Point<f64, Logical>,
    ) {
        if with_pointer_constraint(surface, pointer, |constraint| {
            constraint.is_some_and(|c| c.is_active())
        }) {
            let origin = self
                .space
                .elements()
                .find_map(|window| {
                    (window.wl_surface().as_deref() == Some(surface)).then(|| window.geometry())
                })
                .unwrap_or_default()
                .loc
                .to_f64();

            pointer.set_location(origin + location);
        }
    }
}
delegate_pointer_constraints!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

impl<BackendData: Backend> TabletSeatHandler for FlyjaState<BackendData> {
    fn tablet_tool_image(&mut self, _tool: &TabletToolDescriptor, image: CursorImageStatus) {
        // TODO: tablet tools should have their own cursors
        self.cursor_status = image;
    }
}
delegate_tablet_manager!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);
