use smithay::{
    delegate_xdg_shell,
    //input::{pointer::Focus, Seat},
    wayland::shell::xdg::XdgShellHandler,
};

use crate::{state::ResizeState, FlyJa, shell::WindowElement};

impl XdgShellHandler for FlyJa {
    fn grab(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
        _serial: smithay::utils::Serial,
    ) {
        // TODO:
    }
    fn new_popup(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {
        // TODO:
    }
    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let window = WindowElement::new(surface);
        self.space.map_element(window.clone(), (0, 0), false);

        self.reseize_state = ResizeState::NewTopCreated;
    }

    fn toplevel_destroyed(&mut self, _surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // TODO: resize again
    }
    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.xdg_shell_state
    }
    fn move_request(
        &mut self,
        _surface: smithay::wayland::shell::xdg::ToplevelSurface,
        _seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
        _serial: smithay::utils::Serial,
    ) {
    }
}

delegate_xdg_shell!(FlyJa);
