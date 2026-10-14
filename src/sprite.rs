use crate::core::*;
use crate::query_data::*;
use bevy::ecs::system::lifetimeless::Write;
use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct SpriteColorTween {
    pub start: Color,
    pub end: Color,
}

impl TweenType for SpriteColorTween {
    type Strategy = QueryDataTweenStrategy<Write<Sprite>>;

    fn mutate_subject(
        &self,
        mut item: <<Self::Strategy as TweenStrategy>::Param as TweenSystemParam>::Subject<'_, '_>,
        factor: f32,
    ) {
        item.color = self.start.mix(&self.end, factor);
    }
}
