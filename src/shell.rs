use smithay::{
    desktop::{space::SpaceElement, Window},
    output::Output,
    utils::{IsAlive, Point, Rectangle},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct WindowInfo {
    position: (i32, i32),
    size: (i32, i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowElement {
    window: Window,
    info: WindowInfo,
}

impl IsAlive for WindowElement {
    fn alive(&self) -> bool {
        self.window.alive()
    }
}

impl SpaceElement for WindowElement {
    fn geometry(&self) -> Rectangle<i32, smithay::utils::Logical> {
        todo!()
    }
    fn bbox(&self) -> Rectangle<i32, smithay::utils::Logical> {
        todo!()
    }
    fn is_in_input_region(&self, _point: &Point<f64, smithay::utils::Logical>) -> bool {
        todo!()
    }
    fn z_index(&self) -> u8 {
        SpaceElement::z_index(&self.window)
    }

    fn set_activate(&self, activated: bool) {
        self.window.set_activate(activated)
    }
    fn output_enter(&self, output: &Output, overlap: Rectangle<i32, smithay::utils::Logical>) {
        SpaceElement::output_enter(&self.window, output, overlap)
    }
    fn output_leave(&self, output: &Output) {
        SpaceElement::output_leave(&self.window, output)
    }
    fn refresh(&self) {
        SpaceElement::refresh(&self.window)
    }
}
