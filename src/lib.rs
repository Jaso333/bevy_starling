use bevy_app::{PluginGroup, PluginGroupBuilder};

pub mod asset;
#[cfg(feature = "color_material")]
pub mod color_material;
pub mod core;
pub mod query;
#[cfg(feature = "sprite")]
pub mod sprite;
#[cfg(feature = "text_color")]
pub mod text_color;
#[cfg(feature = "transform")]
pub mod transform;

pub struct TweenPlugins;

impl PluginGroup for TweenPlugins {
    fn build(self) -> PluginGroupBuilder {
        #[allow(unused_mut)]
        let mut builder = PluginGroupBuilder::start::<Self>().add(crate::core::TweenPlugin);

        #[cfg(feature = "transform")]
        {
            use crate::core::GetTweenTypePlugin;
            builder = builder.add(crate::transform::TransformTween::plugin());
        }

        #[cfg(feature = "sprite")]
        {
            use crate::core::GetTweenTypePlugin;
            builder = builder.add(crate::sprite::SpriteTween::plugin());
        }

        #[cfg(feature = "color_material")]
        {
            use crate::core::GetTweenTypePlugin;
            use bevy_sprite_render::{ColorMaterial, MeshMaterial2d};

            builder = builder.add(crate::color_material::ColorMaterialTween::<
                MeshMaterial2d<ColorMaterial>,
            >::plugin());
        }

        #[cfg(feature = "text_color")]
        {
            use crate::core::GetTweenTypePlugin;
            builder = builder.add(crate::text_color::TextColorTween::plugin());
        }

        builder
    }
}
