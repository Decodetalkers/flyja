use std::{ffi::OsString, os::unix::io::AsRawFd, sync::Arc};

use smithay::{
    desktop::{Space, WindowSurfaceType},
    input::Seat,
    input::{pointer::PointerHandle, SeatState},
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, Mode, PostAction},
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{backend::ClientData, protocol::wl_surface::WlSurface, Display},
    },
    utils::{Logical, Point, Size},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        data_device::DataDeviceState,
        output::OutputManagerState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

pub trait Backend {
    const HAS_RELATIVE_MOTION: bool = false;
    fn seat_name(&self) -> String;
}

use crate::{shell::WindowElement, CalloopData};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum WmStatus {
    Tile,
    #[default]
    Stack,
}

#[derive(Debug, Default)]
pub enum PeddingResize {
    ReadyToResize,
    Resizing(WlSurface),
    ResizeFinished(WlSurface),
    #[default]
    Stop,
}

impl WmStatus {
    pub fn status_change(&mut self) {
        match self {
            WmStatus::Tile => *self = WmStatus::Stack,
            WmStatus::Stack => *self = WmStatus::Tile,
        }
    }
}

pub struct FlyJa<BackendData: Backend + 'static> {
    pub backend_data: BackendData,
    pub start_time: std::time::Instant,
    pub socket_name: OsString,

    pub space: Space<WindowElement>,
    pub loop_signal: LoopSignal,

    // State
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<FlyJa<BackendData>>,
    pub pointer: PointerHandle<FlyJa<BackendData>>,
    pub data_device_state: DataDeviceState,

    pub seat: Seat<Self>,
    pub seat_name: String,

    pub reseize_state: PeddingResize,
    pub wmstatus: WmStatus,
}

impl<BackendData: Backend + 'static> FlyJa<BackendData> {
    pub fn init(
        backend_data: BackendData,
        event_loop: &mut EventLoop<CalloopData<BackendData>>,
        display: &mut Display<FlyJa<BackendData>>,
    ) -> Self {
        let start_time = std::time::Instant::now();

        let dh = display.handle();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, Vec::new());
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let mut seat_state = SeatState::new();
        let data_device_state = DataDeviceState::new::<Self>(&dh);

        let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, "winit");

        seat.add_keyboard(Default::default(), 200, 200).unwrap();

        let pointer = seat.add_pointer();

        let space = Space::default();

        let socket_name = Self::init_wayland_listener(display, event_loop);

        let loop_signal = event_loop.get_signal();

        let seat_name = backend_data.seat_name();
        Self {
            backend_data,
            start_time,

            space,
            loop_signal,
            socket_name,

            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,

            seat_state,
            data_device_state,
            seat,
            pointer,
            seat_name,

            reseize_state: PeddingResize::Stop,
            wmstatus: WmStatus::Tile,
        }
    }

    pub fn get_element_count(&self) -> usize {
        self.space.elements().count()
    }

    fn init_wayland_listener<T>(
        display: &mut Display<FlyJa<BackendData>>,
        event_loop: &mut EventLoop<CalloopData<T>>,
    ) -> OsString
    where
        T: Backend + 'static,
    {
        let listening_socket = ListeningSocketSource::new_auto().unwrap();

        let socket_name = listening_socket.socket_name().to_os_string();

        let handle = event_loop.handle();

        event_loop
            .handle()
            .insert_source(listening_socket, move |client_stream, _, state| {
                state
                    .display
                    .handle()
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                    .unwrap();
            })
            .expect("Failed");

        handle
            .insert_source(
                Generic::new(
                    display.backend().poll_fd().as_raw_fd(),
                    Interest::READ,
                    Mode::Level,
                ),
                |_, _, state| {
                    state.display.dispatch_clients(&mut state.state).unwrap();
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();
        socket_name
    }

    fn get_size_and_point(&mut self) -> Option<(i32, i32, i32, i32)> {
        let Some(window) = self.space.elements().find(|w| {
            w.toplevel()
                .current_state()
                .states
                .contains(xdg_toplevel::State::Activated)
        }) else {
            return None;
        };

        let geometry = window.geometry();

        let Some(Point { x, y, .. }) = self.space.element_location(window) else {
            return None;
        };
        let x = x + geometry.size.w / 2;

        let width = geometry.size.w / 2;
        let height = geometry.size.h;
        let surface = window.toplevel();

        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            state.size = Some((width, height).into());
        });
        surface.send_pending_configure();

        Some((x, y, width, height))
    }

    pub fn surface_under_pointer(
        &self,
        pointer: &PointerHandle<Self>,
    ) -> Option<(WlSurface, Point<i32, Logical>)> {
        let pos = pointer.current_location();
        self.space
            .element_under(pos)
            .and_then(|(window, location)| {
                window
                    .surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)
                    .map(|(s, p)| (s, p + location))
            })
    }

    fn handle_one_element(&mut self, surface: &WlSurface) {
        self.reseize_state = PeddingResize::ResizeFinished(surface.clone());
        let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
        else {
            return;
        };
        let prosize = 'block: {
            let Some(output) = self
                .space
                .output_under(self.pointer.current_location())
                .next()
            else {
                break 'block Size::from((1000, 1000));
            };
            let Some(geo) = self.space.output_geometry(output) else {
                break 'block Size::from((1000, 1000));
            };
            geo.size
        };
        let surface_top = window.toplevel();
        surface_top.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            let size = prosize;
            state.size = Some(size);
        });
        surface_top.send_configure();
    }

    fn handle_split_element(&mut self, surface: &WlSurface) {
        self.reseize_state = PeddingResize::ResizeFinished(surface.clone());

        let Some((x, y, width, height)) = self.get_size_and_point() else {
            return;
        };
        let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
        else {
            return;
        };
        let surface = window.toplevel();
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            state.size = Some((width, height).into());
        });
        surface.send_pending_configure();
        self.space.map_element(window.clone(), (x, y), true);
    }

    pub fn handle_resize_tile_window_changing(&mut self) {
        let PeddingResize::Resizing(ref surface) = self.reseize_state else {
            return;
        };
        let count = self.space.elements().count();
        if count == 1 {
            self.handle_one_element(&surface.clone());
        } else {
            self.handle_split_element(&surface.clone());
        }
    }

    pub fn handle_place_stack_to_center(&mut self) {
        if self.wmstatus != WmStatus::Stack {
            return;
        }
        let PeddingResize::ResizeFinished(ref surface) = self.reseize_state else {
            return;
        };
        let Some(output) = self
            .space
            .output_under(self.pointer.current_location())
            .next()
        else {
            return;
        };
        let Some(geo) = self.space.output_geometry(output) else {
            return;
        };

        let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
        else {
            return;
        };
        let gerwindow = window.geometry();
        let pos_x = geo.size.w / 2 - gerwindow.size.w / 2;
        let pox_y = geo.size.h / 2 - gerwindow.size.h / 2;
        self.space
            .map_element(window.clone(), (pos_x, pox_y), false);
    }

    pub fn handle_resize_tile_window_finished(&mut self) {
        let PeddingResize::ResizeFinished(ref surface) = self.reseize_state else {
            return;
        };
        if self.wmstatus != WmStatus::Tile {
            return;
        }
        let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
        else {
            return;
        };
        let surface = window.toplevel();
        surface.with_pending_state(|state| {
            state.states.unset(xdg_toplevel::State::Resizing);
        });
        surface.send_configure();
        self.reseize_state = PeddingResize::Stop;
    }

    pub fn publish_commit(&self) {
        let Some(window) = self.space.elements().next() else {
            return;
        };
        window.toplevel().send_pending_configure();
    }
}

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: smithay::reexports::wayland_server::backend::ClientId) {}
    fn disconnected(
        &self,
        _client_id: smithay::reexports::wayland_server::backend::ClientId,
        _reason: smithay::reexports::wayland_server::backend::DisconnectReason,
    ) {
    }
}
