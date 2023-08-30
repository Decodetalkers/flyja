use crate::FlyJa;

use crate::shell::WindowElement;

use smithay::{
    input::pointer::{GrabStartData, PointerGrab, PointerInnerHandle},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

#[derive(Debug)]
pub struct MoveSurfaceGrab {
    pub start_data: GrabStartData<FlyJa>,
    pub window: WindowElement,
    pub initial_window_location: Point<i32, Logical>,
}

impl PointerGrab<FlyJa> for MoveSurfaceGrab {
    fn motion(
        &mut self,
        data: &mut FlyJa,
        handle: &mut PointerInnerHandle<'_, FlyJa>,
        _focus: Option<(WlSurface, Point<i32, Logical>)>,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        handle.motion(data, None, event);

        let delta = event.location - self.start_data.location;
        let new_location = self.initial_window_location.to_f64() + delta;
        self.window.tileinfo.position = new_location.clone();

        data.space
            .map_element(self.window.clone(), new_location.to_i32_round(), true);
    }

    fn relative_motion(
        &mut self,
        data: &mut FlyJa,
        handle: &mut PointerInnerHandle<'_, FlyJa>,
        focus: Option<(WlSurface, Point<i32, Logical>)>,
        event: &smithay::input::pointer::RelativeMotionEvent,
    ) {
        handle.relative_motion(data, focus, event)
    }

    fn axis(
        &mut self,
        data: &mut FlyJa,
        handle: &mut PointerInnerHandle<'_, FlyJa>,
        details: smithay::input::pointer::AxisFrame,
    ) {
        handle.axis(data, details)
    }

    fn button(
        &mut self,
        data: &mut FlyJa,
        handle: &mut PointerInnerHandle<'_, FlyJa>,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        handle.button(data, event);
        const BTN_LEFT: u32 = 0x110;
        if !handle.current_pressed().contains(&BTN_LEFT) {
            // No more buttons are pressed, release the grab.
            handle.unset_grab(data, event.serial, event.time);
        }
    }

    fn start_data(&self) -> &GrabStartData<FlyJa> {
        &self.start_data
    }
}
