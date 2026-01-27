mod xdg_shell;
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_xdg_shell,
    desktop::Window,
    reexports::wayland_server::{
        Client,
        protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    },
    wayland::{
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState, get_parent,
            is_sync_subsurface,
        },
        shell::xdg::{
            PopupState, PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler,
            XdgShellState,
        },
    },
};

use crate::state::{Backend, ClientState, FlyjaState};

// This implement the compositor
impl<BackendData: Backend> CompositorHandler for FlyjaState<BackendData> {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }
    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self
                .space
                .elements()
                .find(|w| w.toplevel().unwrap().wl_surface() == &root)
            {
                window.on_commit();
            }
        };
        self.handle_xdg_commit(surface);
    }
}

delegate_compositor!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);

// TODO: remove to shell folder
impl<BackendData: Backend> XdgShellHandler for FlyjaState<BackendData> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface.clone());
        self.space.map_element(window, (0, 0), true);
    }
    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {}
    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: smithay::utils::Serial) {}
    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
    }
}

delegate_xdg_shell!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);
