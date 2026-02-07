use smithay::{
    delegate_xdg_shell,
    desktop::PopupKind,
    input::{
        Seat,
        pointer::{Focus, GrabStartData as PointerGrabStartData},
    },
    reexports::{
        wayland_protocols::xdg::decoration as xdg_decoration,
        wayland_server::{
            Resource,
            protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
        },
    },
    utils::Serial,
    wayland::{
        compositor::with_states,
        seat::WaylandFocus,
        shell::xdg::{
            Configure, PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler,
            XdgShellState, XdgToplevelSurfaceData,
        },
    },
};

use flyja_logic::Id;

use crate::{
    grabs::MoveSurfaceGrabSlack,
    shell::WindowElement,
    state::{Backend, FlyjaState, MapMode},
};

impl<BackendData: Backend> FlyjaState<BackendData> {
    pub fn find_window_with_pedding(&self, surface: &WlSurface) -> Option<&WindowElement> {
        let mut window_try = self
            .pedding_windows
            .iter()
            .find(|w| w.wl_surface().as_deref() == Some(surface));
        if window_try.is_none() {
            window_try = self
                .tile_space
                .elements()
                .find(|w| w.wl_surface().as_deref() == Some(surface));
        }
        if window_try.is_none() {
            window_try = self
                .slack_space
                .elements()
                .find(|w| w.wl_surface().as_deref() == Some(surface));
        }
        window_try
    }
    pub fn find_window_in_space(&self, surface: &WlSurface) -> Option<&WindowElement> {
        let mut window_try = self
            .tile_space
            .elements()
            .find(|w| w.wl_surface().as_deref() == Some(surface));
        if window_try.is_none() {
            window_try = self
                .slack_space
                .elements()
                .find(|w| w.wl_surface().as_deref() == Some(surface));
        }
        window_try
    }
    pub fn handle_xdg_commit(&mut self, surface: &WlSurface) {
        let window_try_pedding = self
            .pedding_windows
            .iter()
            .enumerate()
            .find(|(_, w)| w.toplevel().unwrap().wl_surface() == surface)
            .map(|(index, w)| (index, w.clone()));
        let window_try = match window_try_pedding {
            Some((index, window)) => {
                self.pedding_windows.remove(index);

                Some(window)
            }
            None => self.find_window_in_space(surface).cloned(),
        };
        if let Some(window) = window_try {
            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            });

            if !initial_configure_sent {
                window.toplevel().unwrap().send_configure();

                if !window.mapped() {
                    window.state_mut().mapped = true;
                    self.insert_window_new(window);
                }
            }
        }
        self.popups.commit(surface);
        if let Some(popup) = &self.popups.find_popup(surface) {
            match popup {
                PopupKind::Xdg(xdg) => {
                    if !xdg.is_initial_configure_sent() {
                        // NOTE: This should never fail as the initial configure is always
                        // allowed.
                        xdg.send_configure().expect("initial configure failed");
                    }
                }
                PopupKind::InputMethod(_input_method) => {}
            }
        }
    }
}

impl<BackendData: Backend> XdgShellHandler for FlyjaState<BackendData> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let id = Id::unique();
        let window = WindowElement::new_wayland_window(id, self.map_mode, surface.clone());
        self.pedding_windows.push(window);
    }
    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let Some(window) = self
            .tile_space
            .elements()
            .find(|w| w.toplevel().unwrap().wl_surface() == surface.wl_surface())
            .cloned()
        else {
            return;
        };

        self.delete_window(window);
    }
    // TODO: later
    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}
    // TODO: later
    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: smithay::utils::Serial) {}
    // TODO: layter
    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
    }
    fn move_request(&mut self, surface: ToplevelSurface, seat: WlSeat, serial: Serial) {
        let seat = Seat::from_resource(&seat).unwrap();

        let wl_surface = surface.wl_surface();

        if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
            let pointer = seat.get_pointer().unwrap();

            let window = self.find_window_in_space(wl_surface).unwrap().clone();
            if window.mode == MapMode::Tile {
                // TODO: implement later
                return;
            }
            let initial_window_location = self.slack_space.element_location(&window).unwrap();

            let grab = MoveSurfaceGrabSlack {
                start_data,
                window,
                initial_window_location,
            };

            pointer.set_grab(self, grab, serial, Focus::Clear);
        }
    }
    fn ack_configure(&mut self, surface: WlSurface, configure: Configure) {
        let Configure::Toplevel(configure) = configure else {
            return;
        };
        let Some(window) = self.find_window_with_pedding(&surface) else {
            return;
        };
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
        let is_ssd = configure
            .state
            .decoration_mode
            .map(|mode| mode == Mode::ServerSide)
            .unwrap_or(false);
        window.set_ssd(is_ssd);
    }
}

delegate_xdg_shell!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

fn check_grab<BackendData: Backend>(
    seat: &Seat<FlyjaState<BackendData>>,
    surface: &WlSurface,
    serial: Serial,
) -> Option<PointerGrabStartData<FlyjaState<BackendData>>> {
    let pointer = seat.get_pointer()?;

    // Check that this surface has a click grab.
    if !pointer.has_grab(serial) {
        return None;
    }

    let start_data = pointer.grab_start_data()?;

    let (focus, _) = start_data.focus.as_ref()?;
    // If the focus was for a different surface, ignore the request.
    if !focus.id().same_client_as(&surface.id()) {
        return None;
    }

    Some(start_data)
}
