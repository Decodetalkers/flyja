mod compositor;
mod xdg_shell;
use smithay::{
    delegate_data_device, delegate_output, delegate_seat,
    input::SeatHandler,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::data_device::{ClientDndGrabHandler, DataDeviceHandler, ServerDndGrabHandler},
};

use crate::state::FlyJa;

impl SeatHandler for FlyJa {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    fn seat_state(&mut self) -> &mut smithay::input::SeatState<Self> {
        &mut self.seat_state
    }
    fn cursor_image(
        &mut self,
        _seat: &smithay::input::Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }
    fn focus_changed(
        &mut self,
        _seat: &smithay::input::Seat<Self>,
        _focused: Option<&Self::KeyboardFocus>,
    ) {
    }
}
delegate_seat!(FlyJa);

//
// Wl Data Device
//
//
impl DataDeviceHandler for FlyJa {
    type SelectionUserData = ();
    fn data_device_state(&self) -> &smithay::wayland::data_device::DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for FlyJa {}
impl ServerDndGrabHandler for FlyJa {}

delegate_data_device!(FlyJa);

// Wl Output & Xdg Output

delegate_output!(FlyJa);
