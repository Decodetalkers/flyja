use smithay::{
    delegate_xdg_shell,
    desktop::Space,
    reexports::wayland_server::protocol::{wl_seat, wl_surface},
    utils::Serial,
    wayland::{
        compositor::with_states,
        shell::xdg::{ToplevelSurface, XdgShellHandler, XdgToplevelSurfaceData},
    },
};

use crate::{shell::WindowElement, state::ResizeState, FlyJa};

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
    fn move_request(&mut self, _surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
    }
}

pub fn handle_commit(
    space: &mut Space<WindowElement>,
    surface: &wl_surface::WlSurface,
) -> Option<()> {
    let window = space
        .elements()
        .find(|w| w.toplevel().wl_surface() == surface)
        .cloned()?;

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
        window.toplevel().send_configure();
    }

    Some(())
}
delegate_xdg_shell!(FlyJa);
