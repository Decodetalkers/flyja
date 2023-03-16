use smithay::{
    delegate_xdg_shell,
    desktop::Window,
    //input::{pointer::Focus, Seat},
    wayland::shell::xdg::XdgShellHandler,
    utils::{Size, Logical}, reexports::wayland_protocols::xdg::shell::server::xdg_toplevel
};

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
        let window = Window::new(surface);
        //let surface = window.toplevel();
        //surface.with_pending_state(|state| {
        //    state.size = Some(Size::from((1000,1000)));
        //    //state.states.set(xdg_toplevel::State::TiledTop);
        //});
        //surface.send_configure();
        //println!("create a new surface");
        //window.on_commit();
        self.space.map_element(window, (0, 0), false);
    }
    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.xdg_shell_state
    }
    fn move_request(
        &mut self,
        surface: smithay::wayland::shell::xdg::ToplevelSurface,
        seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
        serial: smithay::utils::Serial,
    ) {
    }
}

delegate_xdg_shell!(FlyJa);
