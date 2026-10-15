use crate::core::*;
use crate::query::*;
use bevy::ecs::system::lifetimeless::Write;
use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct TransformTween;

impl TweenType for TransformTween {
    type Param = QueryTweenParam<'static, 'static, Write<Transform>>;
}

#[derive(Default, Clone)]
pub struct TransformTranslationTween {
    pub start: Vec3,
    pub end: Vec3,
}

impl TweenMutatorImpl for TransformTranslationTween {
    type Subject = QueryTweenSubject<Write<Transform>>;

    fn mutate(&self, mut subject: Mut<'_, Transform>, factor: f32) {
        subject.translation = self.start.lerp(self.end, factor);
    }
}

#[derive(Clone)]
pub struct TransformRotationTween {
    pub axis: Dir3,
    pub start: f32,
    pub end: f32,
}

impl Default for TransformRotationTween {
    fn default() -> Self {
        Self {
            axis: Dir3::NEG_Z,
            start: default(),
            end: default(),
        }
    }
}

impl TweenMutatorImpl for TransformRotationTween {
    type Subject = QueryTweenSubject<Write<Transform>>;

    fn mutate(&self, mut subject: Mut<'_, Transform>, factor: f32) {
        subject.rotation =
            Quat::from_axis_angle(self.axis.as_vec3(), self.start.lerp(self.end, factor));
    }
}
