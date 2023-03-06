use smithay::{delegate_xdg_shell, wayland::shell::xdg::XdgShellHandler};

use crate::FlyJa;

impl XdgShellHandler for FlyJa {
    fn grab(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
        serial: smithay::utils::Serial,
    ) {
        // TODO:
    }
    fn new_popup(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {
        // TODO:
    }
    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // TODO:
    }
    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.xdg_shell_state
    }
}

delegate_xdg_shell!(FlyJa);
