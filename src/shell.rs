use std::time::Duration;

use smithay::{
    backend::renderer::{
        element::{
            solid::SolidColorRenderElement, surface::WaylandSurfaceRenderElement, AsRenderElements,
        },
        ImportAll, ImportMem, Renderer,
    },
    desktop::{space::SpaceElement, Space, Window, WindowSurfaceType},
    output::Output,
    reexports::wayland_server::protocol::wl_surface,
    render_elements,
    utils::{IsAlive, Logical, Physical, Point, Rectangle, Scale, Size},
    wayland::{compositor::SurfaceData, shell::xdg::ToplevelSurface},
};

#[derive(Debug, Clone)]
pub struct WindowElement {
    window: Window,
    pub resize_size: Option<((i32, i32), (i32, i32))>,
}

impl PartialEq for WindowElement {
    fn eq(&self, other: &Self) -> bool {
        self.window == other.window
    }
}

impl WindowElement {
    pub fn remap_element(&self, space: &mut Space<Self>) {
        let window = WindowElement {
            resize_size: None,
            ..self.clone()
        };
        let Some(position) = space.element_location(self) else {
            return;
        };
        space.map_element(window, position, true);
    }

    pub fn is_resize_finished(&self, space: &Space<Self>) -> bool {
        let Some(((start_x, start_y), (end_x, end_y))) = self.resize_size else {
            return false;
        };
        let Some(Point { x, y, .. }) = space.element_location(self) else {
            return false;
        };

        let Size { w, h, .. } = self.geometry().size;
        (start_x - x).abs() < 5
            && (start_y - y).abs() < 5
            && (x + w - end_x).abs() < 5
            && (y + h - end_y).abs() < 5
    }

    pub fn set_resize_size(&self, resize_size: ((i32, i32), (i32, i32))) -> Self {
        WindowElement {
            resize_size: Some(resize_size),
            ..self.clone()
        }
    }

    pub fn get_pedding_size(&self) -> (i32, i32) {
        if let Some(((start_x, start_y), (end_x, end_y))) = self.resize_size {
            return (end_x - start_x, end_y - start_y);
        }
        let Size { w, h, .. } = self.geometry().size;
        (w, h)
    }
}

impl WindowElement {
    pub fn new(surface: ToplevelSurface) -> Self {
        WindowElement {
            window: Window::new(surface),
            resize_size: None,
        }
    }

    pub fn toplevel(&self) -> &ToplevelSurface {
        self.window.toplevel()
    }

    pub fn surface_under<P>(
        &self,
        point: P,
        surface_type: WindowSurfaceType,
    ) -> Option<(wl_surface::WlSurface, Point<i32, Logical>)>
    where
        P: Into<Point<f64, Logical>>,
    {
        self.window.surface_under(point, surface_type)
    }

    pub fn on_commit(&self) {
        self.window.on_commit()
    }

    pub fn set_activated(&self, active: bool) -> bool {
        self.window.set_activated(active)
    }

    pub fn send_frame<T, F>(
        &self,
        output: &Output,
        time: T,
        throttle: Option<Duration>,
        primary_scan_out_output: F,
    ) where
        T: Into<Duration>,
        F: FnMut(&wl_surface::WlSurface, &SurfaceData) -> Option<Output> + Copy,
    {
        self.window
            .send_frame(output, time, throttle, primary_scan_out_output)
    }

    pub fn geometry(&self) -> Rectangle<i32, Logical> {
        self.window.geometry()
    }
}

impl IsAlive for WindowElement {
    fn alive(&self) -> bool {
        self.window.alive()
    }
}

impl SpaceElement for WindowElement {
    fn geometry(&self) -> Rectangle<i32, smithay::utils::Logical> {
        SpaceElement::geometry(&self.window)
    }

    fn bbox(&self) -> Rectangle<i32, smithay::utils::Logical> {
        SpaceElement::bbox(&self.window)
    }

    fn is_in_input_region(&self, point: &Point<f64, smithay::utils::Logical>) -> bool {
        SpaceElement::is_in_input_region(&self.window, point)
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

render_elements!(
    pub WindowRenderElement<R> where R: ImportAll + ImportMem;
    Window=WaylandSurfaceRenderElement<R>,
    Decoration=SolidColorRenderElement,
);

impl<R: Renderer> std::fmt::Debug for WindowRenderElement<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Window(arg0) => f.debug_tuple("Window").field(arg0).finish(),
            Self::Decoration(arg0) => f.debug_tuple("Decoration").field(arg0).finish(),
            Self::_GenericCatcher(arg0) => f.debug_tuple("_GenericCatcher").field(arg0).finish(),
        }
    }
}

impl<R> AsRenderElements<R> for WindowElement
where
    R: Renderer + ImportAll,
    <R as Renderer>::TextureId: 'static,
{
    type RenderElement = WaylandSurfaceRenderElement<R>;

    #[profiling::function]
    fn render_elements<C: From<WaylandSurfaceRenderElement<R>>>(
        &self,
        renderer: &mut R,
        location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<C> {
        self.window
            .render_elements(renderer, location, scale, alpha)
    }
}
