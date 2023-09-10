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
    Resizing(WlSurface),
    ResizeFinished(WlSurface),
    ResizeTwoWindowFinished((WlSurface, WlSurface)),
    #[default]
    Stop,
}

#[derive(Debug, Default)]
pub enum WindowRemoved {
    #[default]
    NoState,
    Region {
        pos_start: (i32, i32),
        pos_end: (i32, i32),
    },
    PeddingResizeFinished(WlSurface),
    PeddingMutiResizeFinished(Vec<WlSurface>),
}

#[derive(Debug, Default, Clone, Copy)]
pub enum SplitState {
    #[default]
    H,
    V,
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
    pub splitstate: SplitState,
    pub window_remove_state: WindowRemoved,
}

impl<BackendData: Backend + 'static> FlyJa<BackendData> {
    pub fn find_to_resize_v_down(
        &self,
        (start_x, start_y): (i32, i32),
        (end_x, end_y): (i32, i32),
    ) -> Vec<((i32, i32), WindowElement)> {
        let mut output = Vec::new();
        let Some(window) = self.space.elements().find(|w| {
            let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                return false;
            };
            let Size { w, h, .. } = w.geometry().size;
            x == start_x && y + h == start_y && x + w <= end_x
        }) else {
            return output;
        };
        let Some(Point { x, y, .. }) = self.space.element_location(window) else {
            return output;
        };
        let Size { w, .. } = window.geometry().size;
        output.push(((x, y), window.clone()));
        if x + w == end_x {
            return output;
        }

        let mut others = self.find_to_resize_v_down((start_x + w, start_y), (end_x, end_y));

        if others.is_empty() {
            return Vec::new();
        }

        output.append(&mut others);

        output
    }

    pub fn find_to_resize_v_top(
        &self,
        (start_x, start_y): (i32, i32),
        (end_x, end_y): (i32, i32),
    ) -> Vec<((i32, i32), WindowElement)> {
        let mut output = Vec::new();
        let Some(window) = self.space.elements().find(|w| {
            let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                return false;
            };
            let Size { w, .. } = w.geometry().size;
            x == start_x && y == end_y && x + w <= end_x
        }) else {
            return output;
        };
        let Some(Point { x, .. }) = self.space.element_location(window) else {
            return output;
        };
        let Size { w, .. } = window.geometry().size;
        output.push(((start_x, start_y), window.clone()));
        if x + w == end_x {
            return output;
        }

        let mut others = self.find_to_resize_v_top((start_x + w, start_y), (end_x, end_y));

        if others.is_empty() {
            return Vec::new();
        }

        output.append(&mut others);

        output
    }

    pub fn find_to_resize_h_right(
        &self,
        (start_x, start_y): (i32, i32),
        (end_x, end_y): (i32, i32),
    ) -> Vec<((i32, i32), WindowElement)> {
        let mut output = Vec::new();
        let Some(window) = self.space.elements().find(|window| {
            let Some(Point { x, y, .. }) = self.space.element_location(window) else {
                return false;
            };
            let Size { w, h, .. } = window.geometry().size;
            x + w == start_x && y == start_y && y + h <= end_y
        }) else {
            return output;
        };
        let Some(Point { y, x, .. }) = self.space.element_location(window) else {
            return output;
        };
        let Size { h, .. } = window.geometry().size;
        output.push(((x, start_y), window.clone()));
        if y + h == end_y {
            return output;
        }

        let mut others = self.find_to_resize_h_right((start_x, start_y + h), (end_x, end_y));

        if others.is_empty() {
            return Vec::new();
        }

        output.append(&mut others);

        output
    }

    pub fn find_to_resize_h_left(
        &self,
        (start_x, start_y): (i32, i32),
        (end_x, end_y): (i32, i32),
    ) -> Vec<((i32, i32), WindowElement)> {
        let mut output = Vec::new();
        let Some(window) = self.space.elements().find(|window| {
            let Some(Point { x, y, .. }) = self.space.element_location(window) else {
                return false;
            };
            let Size { h, .. } = window.geometry().size;
            x == end_x && y == start_y && y + h <= end_y
        }) else {
            return output;
        };
        let Some(Point { y, .. }) = self.space.element_location(window) else {
            return output;
        };
        let Size { h, .. } = window.geometry().size;
        output.push(((start_x, start_y), window.clone()));
        if y + h == end_y {
            return output;
        }

        let mut others = self.find_to_resize_h_left((start_x, start_y + h), (end_x, end_y));

        if others.is_empty() {
            return Vec::new();
        }

        output.append(&mut others);

        output
    }
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
            splitstate: SplitState::H,
            window_remove_state: WindowRemoved::NoState,
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

    pub fn set_split_state(&mut self, state: SplitState) {
        self.splitstate = state;
        let Some(window) = self.space.elements().find(|w| {
            w.toplevel()
                .current_state()
                .states
                .contains(xdg_toplevel::State::Activated)
        }) else {
            return;
        };
        let surface = window.toplevel();

        let xdg_state = match state {
            SplitState::H => xdg_toplevel::State::TiledRight,
            SplitState::V => xdg_toplevel::State::TiledBottom,
        };

        surface.with_pending_state(|state| {
            state.states.set(xdg_state);
        });
        surface.send_pending_configure();
    }

    fn get_surface_size_and_point(&mut self) -> Option<(WlSurface, i32, i32, i32, i32)> {
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

        let (x, y, width, height) = match self.splitstate {
            SplitState::H => {
                let x = x + geometry.size.w / 2;

                let width = geometry.size.w / 2;
                let height = geometry.size.h;
                (x, y, width, height)
            }
            SplitState::V => {
                let y = y + geometry.size.h / 2;

                let width = geometry.size.w;
                let height = geometry.size.h / 2;
                (x, y, width, height)
            }
        };
        let surface = window.toplevel();

        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            state.size = Some((width, height).into());
        });
        surface.send_pending_configure();

        Some((surface.wl_surface().clone(), x, y, width, height))
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
        let prosize = {
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
        let Some((surface_before, x, y, width, height)) = self.get_surface_size_and_point() else {
            self.reseize_state = PeddingResize::ResizeFinished(surface.clone());
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
        self.reseize_state =
            PeddingResize::ResizeTwoWindowFinished((surface_before, surface.wl_surface().clone()));

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

    // FIXME: I do not know when I can get the geometry
    pub fn handle_place_stack_to_center(&mut self) {
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
        self.reseize_state = PeddingResize::Stop;
    }

    pub fn handle_resize_tile_split_window_finished(&mut self) {
        let PeddingResize::ResizeTwoWindowFinished((ref surfacea, ref surfaceb)) =
            self.reseize_state
        else {
            return;
        };
        if self.wmstatus != WmStatus::Tile {
            return;
        }
        let Some(windowa) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surfacea)
        else {
            return;
        };
        let Some(windowb) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surfaceb)
        else {
            return;
        };
        let surfacea = windowa.toplevel();
        surfacea.with_pending_state(|state| {
            state.states.unset(xdg_toplevel::State::Resizing);
        });

        let surfaceb = windowb.toplevel();
        surfaceb.with_pending_state(|state| {
            state.states.unset(xdg_toplevel::State::Resizing);
        });
        surfaceb.send_configure();
        self.reseize_state = PeddingResize::Stop;
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

    pub fn handle_window_removed_finished(&mut self) {
        let WindowRemoved::PeddingResizeFinished(ref surface) = self.window_remove_state else {
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
            state.states.unset(xdg_toplevel::State::Resizing);
        });
        surface.send_configure();
        self.window_remove_state = WindowRemoved::NoState;
    }

    pub fn handle_window_removed_mul(&mut self) {
        let WindowRemoved::Region { pos_start, pos_end } = self.window_remove_state else {
            return;
        };
        let (elements_and_poss, state) = 'surface: {
            let surfacesa = self.find_to_resize_v_down(pos_start, pos_end);
            if !surfacesa.is_empty() {
                break 'surface (surfacesa, 0);
            }
            let surfaceb = self.find_to_resize_v_top(pos_start, pos_end);
            if !surfaceb.is_empty() {
                break 'surface (surfaceb, 1);
            }
            let surfacec = self.find_to_resize_h_left(pos_start, pos_end);
            if !surfacec.is_empty() {
                break 'surface (surfacec, 2);
            }
            (self.find_to_resize_h_right(pos_start, pos_end), 3)
        };
        for ((start_x, start_y), window) in elements_and_poss.iter() {
            let Size { w, h, .. } = window.geometry().size;
            let height_add = pos_end.1 - pos_start.1;
            let width_add = pos_end.0 - pos_start.0;
            let surface = window.toplevel();
            let size = match state {
                0 | 1 => (w, h + height_add).into(),
                2 | 3 => (w + width_add, h).into(),
                _ => unreachable!(),
            };
            surface.with_pending_state(|state| {
                state.states.set(xdg_toplevel::State::Resizing);
                state.size = Some(size);
            });
            surface.send_pending_configure();
            self.space
                .map_element(window.clone(), (*start_x, *start_y), true);
        }
        let surfaces: Vec<WlSurface> = elements_and_poss
            .iter()
            .map(|e| e.1.toplevel().wl_surface().clone())
            .collect();
        self.window_remove_state = WindowRemoved::PeddingMutiResizeFinished(surfaces);
    }

    pub fn handle_window_mul_removed_finished(&mut self) {
        let WindowRemoved::PeddingMutiResizeFinished(ref surfaces) = self.window_remove_state
        else {
            return;
        };
        for surface in surfaces {
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
        }
        self.window_remove_state = WindowRemoved::NoState;
    }
    pub fn handle_window_removed(&mut self) {
        let WindowRemoved::Region { pos_start, pos_end } = self.window_remove_state else {
            return;
        };
        let mut window: Option<WindowElement> = None;
        if let Some(window_a) = self
            .space
            .elements()
            .find(|window| window.is_to_resize_v_down(pos_start, pos_end, &self.space))
        {
            let Size { w, h, .. } = window_a.geometry().size;
            let height_add = pos_end.1 - pos_start.1;
            let surface = window_a.toplevel();
            surface.with_pending_state(|state| {
                state.states.set(xdg_toplevel::State::Resizing);
                state.size = Some((w, h + height_add).into());
            });
            surface.send_pending_configure();
            self.window_remove_state =
                WindowRemoved::PeddingResizeFinished(surface.wl_surface().clone());
            return;
        }

        if let Some(window_a) = self
            .space
            .elements()
            .find(|window| window.is_to_resize_h_left(pos_start, pos_end, &self.space))
        {
            let Size { w, h, .. } = window_a.geometry().size;
            let width_add = pos_end.0 - pos_start.0;
            let surface = window_a.toplevel();
            surface.with_pending_state(|state| {
                state.states.set(xdg_toplevel::State::Resizing);
                state.size = Some((w + width_add, h).into());
            });
            surface.send_pending_configure();
            self.window_remove_state =
                WindowRemoved::PeddingResizeFinished(surface.wl_surface().clone());
            return;
        }

        if let Some(window_a) = self
            .space
            .elements()
            .find(|window| window.is_to_resize_v_top(pos_start, pos_end, &self.space))
        {
            window = Some(window_a.clone());
            let Size { w, h, .. } = window_a.geometry().size;
            let height_add = pos_end.1 - pos_start.1;
            let surface = window_a.toplevel();
            surface.with_pending_state(|state| {
                state.states.set(xdg_toplevel::State::Resizing);
                state.size = Some((w, h + height_add).into());
            });
            surface.send_pending_configure();
            self.window_remove_state =
                WindowRemoved::PeddingResizeFinished(surface.wl_surface().clone());
        } else if let Some(window_a) = self
            .space
            .elements()
            .find(|window| window.is_to_resize_h_right(pos_start, pos_end, &self.space))
        {
            window = Some(window_a.clone());
            let Size { w, h, .. } = window_a.geometry().size;
            let width_add = pos_end.0 - pos_start.0;
            let surface = window_a.toplevel();
            surface.with_pending_state(|state| {
                state.states.set(xdg_toplevel::State::Resizing);
                state.size = Some((w + width_add, h).into());
            });
            self.window_remove_state =
                WindowRemoved::PeddingResizeFinished(surface.wl_surface().clone());
        }
        if let Some(window) = window {
            self.space.map_element(window, pos_start, true);
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
