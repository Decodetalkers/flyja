use smithay::{
    delegate_xdg_shell,
    desktop::Space,
    input::{
        pointer::{Focus, GrabStartData},
        Seat,
    },
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            protocol::{wl_seat, wl_surface},
            Resource,
        },
    },
    utils::{Point, Serial},
    wayland::{
        compositor::with_states,
        shell::xdg::{Configure, ToplevelSurface, XdgShellHandler, XdgToplevelSurfaceData},
    },
};

use crate::{grab::move_grab::MoveSurfaceGrab, shell::WindowElement, FlyJa, state::PeddingResize};

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
        let window = WindowElement::new(surface, Point::from((0.0, 0.0)));
        self.space.map_element(window.clone(), (0, 0), false);

        self.reseize_state = PeddingResize::ReadyToResize;
    }

    fn toplevel_destroyed(&mut self, _surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // TODO: resize again
    }

    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.xdg_shell_state
    }

    fn resize_request(
        &mut self,
        _surface: ToplevelSurface,
        _seat: wl_seat::WlSeat,
        _serial: Serial,
        _edges: xdg_toplevel::ResizeEdge,
    ) {
        println!("ffffffff");
    }

    fn move_request(&mut self, surface: ToplevelSurface, seat: wl_seat::WlSeat, serial: Serial) {
        let seat: Seat<FlyJa> = Seat::from_resource(&seat).unwrap();
        let wl_surface = surface.wl_surface();
        let Some(start_data) = check_grab(&seat, wl_surface, serial) else {
            return;
        };
        let pointer = seat.get_pointer().unwrap();
        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == wl_surface)
            .unwrap()
            .clone();

        let initial_window_location = self.space.element_location(&window).unwrap();

        let grab = MoveSurfaceGrab {
            start_data,
            window,
            initial_window_location,
        };

        pointer.set_grab(self, grab, serial, Focus::Clear);
    }

    fn ack_configure(&mut self, _surface: wl_surface::WlSurface, configure: Configure) {
        if let Configure::Toplevel(configure) = configure {
            println!("{:?}", configure.state);
        }
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

fn check_grab(
    seat: &Seat<FlyJa>,
    surface: &wl_surface::WlSurface,
    serial: Serial,
) -> Option<GrabStartData<FlyJa>> {
    let pointer = seat.get_pointer()?;

    if !pointer.has_grab(serial) {
        return None;
    }

    let start_data = pointer.grab_start_data()?;
    let (focus, _) = start_data.focus.as_ref()?;
    if !focus.id().same_client_as(&surface.id()) {
        return None;
    }
    Some(start_data)
}
