use crate::asset::*;
use crate::core::*;
use bevy_asset::prelude::*;
use bevy_color::prelude::*;
use bevy_ecs::prelude::*;
use bevy_sprite_render::prelude::*;
use std::marker::PhantomData;
use std::ops::Deref;

pub struct ColorMaterialTween<R> {
    _phantom: PhantomData<fn() -> R>,
}

impl<R: Component + Deref<Target = Handle<ColorMaterial>>> TweenType for ColorMaterialTween<R> {
    type Param = AssetTweenParam<'static, 'static, ColorMaterial, R>;
}

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
