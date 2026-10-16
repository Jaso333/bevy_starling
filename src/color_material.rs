use crate::asset::*;
use crate::core::*;
use bevy::prelude::*;
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

impl TweenMutatorImpl for ColorMaterialColorTween {
    type Subject = AssetTweenSubject<ColorMaterial>;

    fn mutate(&self, mut subject: <Self::Subject as TweenSubject>::Item<'_, '_>, factor: f32) {
        subject.color = self.start.mix(&self.end, factor)
    }
}
