use crate::asset::*;
use crate::core::*;
use bevy_asset::prelude::*;
use bevy_color::prelude::*;
use bevy_ecs::prelude::*;
use bevy_sprite_render::prelude::*;
use std::marker::PhantomData;
use std::ops::Deref;

/// Tweens the [`ColorMaterial`] referenced by component `R`. Component `R` must implement [`Deref<Target = Handle<ColorMaterial>>`]
pub struct ColorMaterialTween<R> {
    _phantom: PhantomData<fn() -> R>,
}

impl<R: Component + Deref<Target = Handle<ColorMaterial>>> TweenType for ColorMaterialTween<R> {
    type Param = AssetTweenParam<'static, 'static, ColorMaterial, R>;
}

/// Mutates [`ColorMaterial::color`] between the `start` and `end` values.
///
/// Using HSV colors may not yield desirable results due to the way that HSV color interpolation uses "shortest path" logic.
#[derive(Clone, Default)]
pub struct ColorMaterialColorTween {
    pub start: Color,
    pub end: Color,
}

impl TweenMutator for ColorMaterialColorTween {
    type Subject = AssetTweenSubject<ColorMaterial>;

    fn mutate(&self, subject: &mut <Self::Subject as TweenSubject>::Item<'_, '_>, factor: f32) {
        subject.color = self.start.mix(&self.end, factor)
    }
}
