use smithay::{
    delegate_xdg_shell,
    desktop::Window,
    utils::Size, // utils::{Size, Logical}, reexports::wayland_protocols::xdg::shell::server::xdg_toplevel
    //input::{pointer::Focus, Seat},
    wayland::shell::xdg::XdgShellHandler,
};

use crate::FlyJa;

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
        let window = Window::new(surface);
        self.space.map_element(window.clone(), (100, 100), false);

        for window in self.space.elements() {
            let surface = window.toplevel();
            surface.with_pending_state(|state| {
                let size = Size::from((3000, 3000));
                state.size = Some(size);
            });
            surface.send_configure();
        }
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
