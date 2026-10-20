use crate::core::*;
use crate::query::*;
use bevy_color::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::lifetimeless::Write;
use bevy_sprite::prelude::*;

pub struct SpriteTween;

impl TweenType for SpriteTween {
    type Param = QueryTweenParam<'static, 'static, Write<Sprite>>;
}

#[derive(Clone, Default)]
pub struct SpriteColorTween {
    pub start: Color,
    pub end: Color,
}

impl TweenMutator for SpriteColorTween {
    type Subject = QueryTweenSubject<Write<Sprite>>;

    fn mutate(&self, subject: &mut Mut<'_, Sprite>, factor: f32) {
        subject.color = self.start.mix(&self.end, factor);
    }
}
