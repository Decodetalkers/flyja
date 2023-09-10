use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_shm,
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Client},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            get_parent, is_sync_subsurface, CompositorClientState, CompositorHandler,
            CompositorState,
        },
        shm::ShmHandler,
    },
};

use crate::{
    state::{Backend, ClientState, WmStatus},
    FlyJa,
};

impl<BackendData: Backend> CompositorHandler for FlyJa<BackendData> {
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
                .find(|w| w.toplevel().wl_surface() == &root)
            {
                window.on_commit();
            }
        }

        self.handle_commit(surface);

        self.handle_window_removed_mul();
        self.handle_window_mul_removed_finished();

        // TODO: need know the geo before put it to center
        // if self.wmstatus == WmStatus::Stack {
        //     self.handle_place_stack_to_center();
        // }

        // Tile
        if self.wmstatus == WmStatus::Tile {
            self.handle_resize_tile_window_changing();
            self.handle_resize_tile_window_finished();
            self.handle_resize_tile_split_window_finished();
        }
    }
}

impl<BackendData: Backend> BufferHandler for FlyJa<BackendData> {
    fn buffer_destroyed(
        &mut self,
        _buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
    }
}

impl<BackendData: Backend> ShmHandler for FlyJa<BackendData> {
    fn shm_state(&self) -> &smithay::wayland::shm::ShmState {
        &self.shm_state
    }
}

delegate_compositor!(@<BackendData: Backend + 'static> FlyJa<BackendData>);
delegate_shm!(@<BackendData: Backend + 'static> FlyJa<BackendData>);
