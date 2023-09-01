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

use crate::{shell::WindowElement, CalloopData};

#[derive(Debug, Default)]
pub enum WmStatus {
    Tile,
    TiteToStack,
    #[default]
    Stack,
    StackToTite,
}

#[derive(Debug, Default)]
pub enum PeddingResize {
    ReadyToResize,
    Resizing(WlSurface),
    #[default]
    Stop,
}


impl WmStatus {
    pub fn status_change(&mut self) {
        match self {
            WmStatus::Tile => *self = WmStatus::TiteToStack,
            WmStatus::TiteToStack => *self = WmStatus::Stack,
            WmStatus::Stack => *self = WmStatus::StackToTite,
            WmStatus::StackToTite => *self = WmStatus::Tile,
        }
    }
    pub fn is_changing(&self) -> bool {
        matches!(self, WmStatus::StackToTite | WmStatus::TiteToStack)
    }
}

pub struct FlyJa {
    pub start_time: std::time::Instant,
    pub socket_name: OsString,

    pub space: Space<WindowElement>,
    pub loop_signal: LoopSignal,

    // State
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<FlyJa>,
    pub data_device_state: DataDeviceState,

    pub seat: Seat<Self>,

    pub reseize_state: PeddingResize,
    pub wmstatus: WmStatus,
}

impl FlyJa {
    pub fn new(event_loop: &mut EventLoop<CalloopData>, display: &mut Display<FlyJa>) -> Self {
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

        seat.add_pointer();

        let space = Space::default();

        let socket_name = Self::init_wayland_listener(display, event_loop);

        let loop_signal = event_loop.get_signal();

        Self {
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
            reseize_state: PeddingResize::Stop,
            wmstatus: WmStatus::Stack,
        }
    }

    pub fn get_element_count(&self) -> usize {
        self.space.elements().count()
    }

    fn init_wayland_listener(
        display: &mut Display<FlyJa>,
        event_loop: &mut EventLoop<CalloopData>,
    ) -> OsString {
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

    pub fn handle_resize_event(&mut self) {
        let PeddingResize::Resizing(ref surface) = self.reseize_state else {
            return;
        };
        let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
        else {
            return;
        };
        println!("ffffffff");
        let surface = window.toplevel();
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            let size = Size::from((1000, 1000));
            state.size = Some(size);
        });
        surface.send_configure();
        self.reseize_state = PeddingResize::Stop;
    }

    pub fn handle_state_change_event(&mut self) {
        if !self.wmstatus.is_changing() {
            return;
        }

        self.wmstatus.status_change();
        let elements: Vec<_> = self.space.elements().cloned().collect();
        for element in elements {
            self.space.map_element(element, (0, 0), false);
        }
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
