use crate::{
    shell::WindowElement,
    state::{Backend, FlyJa},
};

use smithay::input::pointer::{GrabStartData, PointerGrab};

#[derive(Debug)]
pub enum ResizeType {
    LR,
    HV,
}

#[derive(Debug)]
pub struct ResizeSurfaceGrab<BackendData: Backend + 'static> {
    start_data: GrabStartData<FlyJa<BackendData>>,

    pined_elements: Vec<WindowElement>,
    moved_elements: Vec<WindowElement>,

    resize_type: ResizeType,
}

impl<BackendData: Backend> PointerGrab<FlyJa<BackendData>> for ResizeSurfaceGrab<BackendData> {
    fn start_data(&self) -> &GrabStartData<FlyJa<BackendData>> {
        &self.start_data
    }
    fn axis(
        &mut self,
        data: &mut FlyJa<BackendData>,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, FlyJa<BackendData>>,
        details: smithay::input::pointer::AxisFrame,
    ) {
        handle.axis(data, details)
    }

    fn motion(
        &mut self,
        data: &mut FlyJa<BackendData>,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, FlyJa<BackendData>>,
        _focus: Option<(
            <FlyJa<BackendData> as smithay::input::SeatHandler>::PointerFocus,
            smithay::utils::Point<i32, smithay::utils::Logical>,
        )>,
        event: &smithay::input::pointer::MotionEvent,
    ) {
    }
    fn relative_motion(
        &mut self,
        data: &mut FlyJa<BackendData>,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, FlyJa<BackendData>>,
        focus: Option<(
            <FlyJa<BackendData> as smithay::input::SeatHandler>::PointerFocus,
            smithay::utils::Point<i32, smithay::utils::Logical>,
        )>,
        event: &smithay::input::pointer::RelativeMotionEvent,
    ) {
        handle.relative_motion(data, focus, event)
    }
    // TODO:
    fn button(
        &mut self,
        data: &mut FlyJa<BackendData>,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, FlyJa<BackendData>>,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        const BTN_LEFT: u32 = 0x110;
        if !handle.current_pressed().contains(&BTN_LEFT) {
            // No more buttons are pressed, release the grab.
            handle.unset_grab(data, event.serial, event.time);
        }
    }
}
