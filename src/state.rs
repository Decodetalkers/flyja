use smithay::{
    delegate_output,
    input::{Seat, SeatHandler, SeatState},
    reexports::wayland_server::{
        backend::{ClientData, ClientId, DisconnectReason},
        protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorState},
        output::OutputHandler,
        selection::{
            SelectionHandler, SelectionSource, SelectionTarget,
            data_device::{DataDeviceHandler, DataDeviceState, WaylandDndGrabHandler},
        },
    },
};

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}
impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        println!("initialized");
    }

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        println!("disconnected");
    }
}

pub trait Backend {
    const HAS_RELATIVE_MOTION: bool = false;
    fn seat_name(&self) -> String;
}

pub struct FlyjaState<BackendData: Backend + 'static> {
    pub backend_data: BackendData,

    // smithay state
    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub seat_state: SeatState<FlyjaState<BackendData>>,
}

impl<BackendData: Backend> DataDeviceHandler for FlyjaState<BackendData> {
    fn data_device_state(&mut self) -> &mut DataDeviceState {
        &mut self.data_device_state
    }
}

impl<BackendData: Backend> WaylandDndGrabHandler for FlyjaState<BackendData> {}

impl<BackendData: Backend> SeatHandler for FlyjaState<BackendData> {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;
    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }
    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&Self::KeyboardFocus>) {}
}

impl<BackendData: Backend> SelectionHandler for FlyjaState<BackendData> {
    type SelectionUserData = ();
}

impl<BackendData: Backend + 'static> FlyjaState<BackendData> {}

impl<BackendData: Backend> BufferHandler for FlyjaState<BackendData> {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl<BackendData: Backend> OutputHandler for FlyjaState<BackendData> {}

delegate_output!(@<BackendData: Backend + 'static> FlyjaState<BackendData>);
