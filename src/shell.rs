use std::time::Duration;

use smithay::{
    backend::renderer::{
        element::{
            solid::SolidColorRenderElement, surface::WaylandSurfaceRenderElement, AsRenderElements,
        },
        ImportAll, ImportMem, Renderer, Texture,
    },
    desktop::{space::SpaceElement, Window, WindowSurfaceType},
    output::Output,
    reexports::wayland_server::protocol::wl_surface,
    render_elements,
    utils::{IsAlive, Logical, Physical, Point, Rectangle, Scale},
    wayland::{compositor::SurfaceData, shell::xdg::ToplevelSurface},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct WindowInfo {
    position: (i32, i32),
    size: (i32, i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowElement {
    window: Window,
    titleinfo: WindowInfo,
    normalinfo: WindowInfo,
}

impl WindowElement {
    pub fn new(surface: ToplevelSurface) -> Self {
        WindowElement {
            window: Window::new(surface),
            titleinfo: WindowInfo::default(),
            normalinfo: WindowInfo::default(),
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
    R: Renderer + ImportAll + ImportMem,
    <R as Renderer>::TextureId: Texture + 'static,
{
    type RenderElement = WindowRenderElement<R>;

    fn render_elements<C: From<Self::RenderElement>>(
        &self,
        renderer: &mut R,
        location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<C> {
        AsRenderElements::<R>::render_elements::<WindowRenderElement<R>>(
            &self.window,
            renderer,
            location,
            scale,
            alpha,
        )
        .into_iter()
        .map(C::from)
        .collect()
    }
}