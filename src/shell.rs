use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor,
    reexports::wayland_server::{Client, protocol::wl_surface::WlSurface},
    wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState},
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
    }
}

delegate_compositor!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);
