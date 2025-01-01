use smithay::backend::renderer::element::solid::SolidColorRenderElement;
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::Renderer;

#[thin_delegate::external_trait_def(with_uses = true)]
mod external_trait_def {
    use smithay::backend::renderer::element::{Id, Kind, UnderlyingStorage};
    use smithay::backend::renderer::utils::CommitCounter;
    use smithay::backend::renderer::{ImportAll, ImportMem, Renderer};
    use smithay::utils::{Buffer as BufferCoords, Physical, Point, Rectangle, Scale, Transform};

    #[thin_delegate::register]
    pub trait Element {
        /// Get the unique id of this element
        fn id(&self) -> &Id;
        /// Get the current commit position of this element
        fn current_commit(&self) -> CommitCounter;
        /// Get the location relative to the output
        fn location(&self, scale: Scale<f64>) -> Point<i32, Physical> {
            self.geometry(scale).loc
        }
        /// Get the src of the underlying buffer
        fn src(&self) -> Rectangle<f64, BufferCoords>;
        /// Get the transform of the underlying buffer
        fn transform(&self) -> Transform {
            Transform::Normal
        }
        /// Get the geometry relative to the output
        fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical>;
        /// Get the damage since the provided commit relative to the element
        fn damage_since(
            &self,
            scale: Scale<f64>,
            commit: Option<CommitCounter>,
        ) -> Vec<Rectangle<i32, Physical>> {
            if commit != Some(self.current_commit()) {
                vec![Rectangle::from_loc_and_size(
                    (0, 0),
                    self.geometry(scale).size,
                )]
            } else {
                vec![]
            }
        }
        /// Get the opaque regions of the element relative to the element
        fn opaque_regions(&self, _scale: Scale<f64>) -> Vec<Rectangle<i32, Physical>> {
            vec![]
        }
        /// Returns an alpha value the element should be drawn with regardless of any
        /// already encoded alpha in it's underlying representation.
        fn alpha(&self) -> f32 {
            1.0
        }
        /// Returns the [`Kind`] for this element
        fn kind(&self) -> Kind {
            Kind::default()
        }
    }

    #[thin_delegate::register]
    pub trait RenderElement<R: Renderer>: Element {
        /// Draw this element
        fn draw(
            &self,
            frame: &mut <R as Renderer>::Frame<'_>,
            src: Rectangle<f64, BufferCoords>,
            dst: Rectangle<i32, Physical>,
            damage: &[Rectangle<i32, Physical>],
        ) -> Result<(), R::Error>;

        /// Get the underlying storage of this element, may be used to optimize rendering (eg. drm planes)
        fn underlying_storage(&self, renderer: &mut R) -> Option<UnderlyingStorage> {
            let _ = renderer;
            None
        }
    }
}

#[derive(derive_more::From)]
#[thin_delegate::register]
pub enum WindowRenderElement<R>
where
    R: Renderer,
{
    Window(WaylandSurfaceRenderElement<R>),
    Decoration(SolidColorRenderElement),
}

#[thin_delegate::fill_delegate(external_trait_def = external_trait_def)]
impl<R> smithay::backend::renderer::element::Element for WindowRenderElement<R>
where
    R: smithay::backend::renderer::Renderer,
    <R as smithay::backend::renderer::Renderer>::TextureId: 'static,
    R: ImportAll + ImportMem,
{
}

#[thin_delegate::fill_delegate(external_trait_def = external_trait_def)]
impl<R> smithay::backend::renderer::element::RenderElement<R> for WindowRenderElement<R>
where
    R: smithay::backend::renderer::Renderer,
    <R as smithay::backend::renderer::Renderer>::TextureId: 'static,
    R: ImportAll + ImportMem,
{
}

fn main() {}
