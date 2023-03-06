use smithay::{
    delegate_compositor, delegate_shm,
    wayland::{buffer::BufferHandler, compositor::CompositorHandler, shm::ShmHandler},
};

use crate::FlyJa;

//use super::xdg_shell;

impl CompositorHandler for FlyJa {
    fn compositor_state(&mut self) -> &mut smithay::wayland::compositor::CompositorState {
        &mut self.compositor_state
    }
    fn commit(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) {
        // TODO:
    }
}

impl BufferHandler for FlyJa {
    fn buffer_destroyed(
        &mut self,
        buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
    }
}

impl ShmHandler for FlyJa {
    fn shm_state(&self) -> &smithay::wayland::shm::ShmState {
        &self.shm_state
    }
}

delegate_compositor!(FlyJa);
delegate_shm!(FlyJa);
