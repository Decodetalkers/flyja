use std::{
    cell::{Ref, RefCell, RefMut},
    ops::Deref,
};

use smithay::{
    backend::renderer::{
        ImportAll, ImportMem, Renderer, Texture,
        element::{
            AsRenderElements,
            solid::SolidColorRenderElement,
            surface::WaylandSurfaceRenderElement,
            //utils::{ConstrainScaleBehavior, constrain_render_elements},
        },
    },
    desktop::{Window, WindowSurface, space::SpaceElement},
    output::Output,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    render_elements,
    utils::{IsAlive, Logical, Physical, Point, Rectangle, Scale, Size},
    wayland::shell::xdg::ToplevelSurface,
};

use flyja_logic::{Id, Size as FlSize};

// This storage the size and other information
#[derive(Debug, Clone)]
pub struct WindowState {
    pub is_ssd: bool,
    // This is used when it is shown on tile space
    pub geometry: FlSize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowElement {
    pub id: Id,
    window: Window,
}

impl Deref for WindowElement {
    type Target = Window;
    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl WindowElement {
    pub fn new_wayland_window(id: Id, toplevel: ToplevelSurface) -> Self {
        Self {
            id,
            window: Window::new_wayland_window(toplevel),
        }
    }

    pub fn state(&self) -> Ref<'_, WindowState> {
        self.user_data().insert_if_missing(|| {
            RefCell::new(WindowState {
                is_ssd: false,
                geometry: FlSize {
                    width: 0.,
                    height: 0.,
                },
            })
        });
        self.user_data()
            .get::<RefCell<WindowState>>()
            .unwrap()
            .borrow()
    }

    pub fn state_mut(&self) -> RefMut<'_, WindowState> {
        self.user_data().insert_if_missing(|| {
            RefCell::new(WindowState {
                is_ssd: false,
                geometry: FlSize {
                    width: 0.,
                    height: 0.,
                },
            })
        });
        self.user_data()
            .get::<RefCell<WindowState>>()
            .unwrap()
            .borrow_mut()
    }
    pub fn set_geometry(&self, size: FlSize) {
        self.state_mut().geometry = size;
    }

    pub fn set_ssd(&self, is_ssd: bool) {
        self.state_mut().is_ssd = is_ssd;
    }

    pub fn resize(&self, size: FlSize) {
        let WindowSurface::Wayland(surface) = self.underlying_surface() else {
            return;
        };
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            state.size = Some(Size::new(size.width as i32, size.height as i32));
        });
        surface.send_pending_configure();
    }
}

impl IsAlive for WindowElement {
    #[inline]
    fn alive(&self) -> bool {
        self.window.alive()
    }
}

impl SpaceElement for WindowElement {
    fn geometry(&self) -> Rectangle<i32, Logical> {
        let mut geometry = SpaceElement::geometry(&self.window);
        let geometry_saved = self.state().geometry;
        geometry.size.w = geometry_saved.width as i32;
        geometry.size.h = geometry_saved.height as i32;
        geometry
    }
    fn bbox(&self) -> Rectangle<i32, Logical> {
        SpaceElement::bbox(&self.window)
    }
    fn is_in_input_region(&self, point: &Point<f64, Logical>) -> bool {
        SpaceElement::is_in_input_region(&self.window, point)
    }
    fn z_index(&self) -> u8 {
        SpaceElement::z_index(&self.window)
    }

    fn set_activate(&self, activated: bool) {
        SpaceElement::set_activate(&self.window, activated);
    }
    fn output_enter(&self, output: &Output, overlap: Rectangle<i32, Logical>) {
        SpaceElement::output_enter(&self.window, output, overlap);
    }
    fn output_leave(&self, output: &Output) {
        SpaceElement::output_leave(&self.window, output);
    }
    #[profiling::function]
    fn refresh(&self) {
        SpaceElement::refresh(&self.window);
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
    R::TextureId: Clone + Texture + 'static,
{
    type RenderElement = WindowRenderElement<R>;

    fn render_elements<C: From<Self::RenderElement>>(
        &self,
        renderer: &mut R,
        location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<C> {
        AsRenderElements::render_elements(&self.window, renderer, location, scale, alpha)
            .into_iter()
            .map(C::from)
            .collect()
    }
}
