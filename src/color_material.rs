use crate::asset::*;
use crate::core::*;
use bevy::prelude::*;
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Component)]
pub struct ColorMaterialTween<R> {
    pub mutator: ColorMaterialTweenMutator,
    _phantom: PhantomData<fn() -> R>,
}

impl<R> Clone for ColorMaterialTween<R> {
    fn clone(&self) -> Self {
        Self {
            mutator: self.mutator.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<R> Default for ColorMaterialTween<R> {
    fn default() -> Self {
        Self {
            mutator: default(),
            _phantom: PhantomData,
        }
    }
}

impl<R: Component + Deref<Target = Handle<ColorMaterial>>> TweenType for ColorMaterialTween<R> {
    type Strategy = AssetTweenStrategy<ColorMaterial, R>;

    fn mutate_subject(
        &self,
        mut subject: <<Self::Strategy as TweenStrategy>::Param as TweenSystemParam>::Subject<
            '_,
            '_,
        >,
        factor: f32,
    ) {
        match &self.mutator {
            ColorMaterialTweenMutator::Color { start, end } => {
                subject.color = start.mix(end, factor)
            }
        }
    }
}

#[derive(Clone)]
pub enum ColorMaterialTweenMutator {
    Color { start: Color, end: Color },
}

impl Default for ColorMaterialTweenMutator {
    fn default() -> Self {
        Self::Color {
            start: default(),
            end: default(),
        }
    }
}
