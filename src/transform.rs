use crate::core::*;
use crate::query::*;

use bevy_app::PostUpdate;
use bevy_ecs::prelude::*;
use bevy_ecs::system::lifetimeless::Write;
use bevy_math::prelude::*;
use bevy_transform::TransformSystems;
use bevy_transform::components::Transform;

/// Tweens values within the [`Transform`] component. This simply uses [`QueryTweenParam`] to get [`Write<Transform>`] as the subject.
pub struct TransformTween;

impl TweenType for TransformTween {
    type Param = QueryTweenParam<'static, 'static, Write<Transform>>;

    fn configure_app(app: &mut bevy_app::App) {
        // the mutation happens in post-update, so make sure it happens before transform propagation
        app.configure_sets(
            PostUpdate,
            TweenTypeSystems::<Self>::default().before(TransformSystems::Propagate),
        );
    }
}

/// Mutates [`Transform::translation`] between the `start` and `end` values.
#[derive(Default, Clone)]
pub struct TransformTranslationTween {
    pub start: Vec3,
    pub end: Vec3,
}

impl TweenMutator for TransformTranslationTween {
    type Subject = QueryTweenSubject<Write<Transform>>;

    fn mutate(&self, subject: &mut Mut<'_, Transform>, factor: f32) {
        subject.translation = self.start.lerp(self.end, factor);
    }
}

/// Mutates [`Transform::rotation`] between the `start` and `end` values about the `axis` provided.
///
/// This is method of rotation is typically more desirable than [`Quat`] interpolation.
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
            start: Default::default(),
            end: Default::default(),
        }
    }
}

impl TweenMutator for TransformRotationTween {
    type Subject = QueryTweenSubject<Write<Transform>>;

    fn mutate(&self, subject: &mut Mut<'_, Transform>, factor: f32) {
        subject.rotation =
            Quat::from_axis_angle(self.axis.as_vec3(), self.start.lerp(self.end, factor));
    }
}
