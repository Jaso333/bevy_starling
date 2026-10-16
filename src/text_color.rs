use bevy::{ecs::system::lifetimeless::Write, prelude::*};

use crate::{
    core::{TweenMutatorImpl, TweenType},
    query::{QueryTweenParam, QueryTweenSubject},
};

/// Because this is a newtype component, it makes sense to implement both the overall tween type and the mutator itself.
#[derive(Clone, Default)]
pub struct TextColorTween {
    pub start: Color,
    pub end: Color,
}

impl TweenType for TextColorTween {
    type Param = QueryTweenParam<'static, 'static, Write<TextColor>>;
}

impl TweenMutatorImpl for TextColorTween {
    type Subject = QueryTweenSubject<Write<TextColor>>;

    fn mutate(&self, mut subject: Mut<'_, TextColor>, factor: f32) {
        subject.0 = self.start.mix(&self.end, factor);
    }
}
