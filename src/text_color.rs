use crate::{
    core::{TweenMutator, TweenType},
    query::{QueryTweenParam, QueryTweenSubject},
};
use bevy_color::prelude::*;
use bevy_ecs::{prelude::*, system::lifetimeless::Write};
use bevy_text::prelude::*;

/// Tweens [`TextColor`] component between the `start` and `end` values.
///
/// Because this is a newtype component, it makes sense to implement both the overall tween type and the mutator itself.
///
/// Using HSV colors may not yield desirable results due to the way that HSV color interpolation uses "shortest path" logic.
#[derive(Clone, Default)]
pub struct TextColorTween {
    pub start: Color,
    pub end: Color,
}

impl TweenType for TextColorTween {
    type Param = QueryTweenParam<'static, 'static, Write<TextColor>>;
}

impl TweenMutator for TextColorTween {
    type Subject = QueryTweenSubject<Write<TextColor>>;

    fn mutate(&self, subject: &mut Mut<'_, TextColor>, factor: f32) {
        subject.0 = self.start.mix(&self.end, factor);
    }
}
