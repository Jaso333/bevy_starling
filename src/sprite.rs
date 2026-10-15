use crate::core::*;
use crate::query::*;
use bevy::ecs::system::lifetimeless::Write;
use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct SpriteTween;

impl TweenType for SpriteTween {
    type Param = QueryTweenParam<'static, 'static, Write<Sprite>>;
}

#[derive(Clone, Default)]
pub struct SpriteColorTween {
    pub start: Color,
    pub end: Color,
}

impl TweenMutatorImpl for SpriteColorTween {
    type Subject = QueryTweenSubject<Write<Sprite>>;

    fn mutate(&self, mut subject: Mut<'_, Sprite>, factor: f32) {
        subject.color = self.start.mix(&self.end, factor);
    }
}
