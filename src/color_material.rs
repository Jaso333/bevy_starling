use crate::asset::*;
use crate::core::*;
use bevy::prelude::*;
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Component)]
pub struct ColorMaterialColorTween<R: 'static> {
    pub start: Color,
    pub end: Color,
    _phantom: PhantomData<fn() -> R>,
}

impl<R: Component + Deref<Target = Handle<ColorMaterial>>> TweenType
    for ColorMaterialColorTween<R>
{
    type Strategy = AssetTweenStrategy<ColorMaterial, R>;

    fn mutate_subject(
        &self,
        mut item: <<Self::Strategy as TweenStrategy>::Param as TweenSystemParam>::Subject<'_, '_>,
        factor: f32,
    ) {
        item.color = self.start.mix(&self.end, factor);
    }
}

impl<R> Clone for ColorMaterialColorTween<R> {
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            end: self.end,
            _phantom: PhantomData,
        }
    }
}

impl<R> Default for ColorMaterialColorTween<R> {
    fn default() -> Self {
        Self {
            start: default(),
            end: default(),
            _phantom: PhantomData,
        }
    }
}
