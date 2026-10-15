use crate::core::*;
use crate::query_data::*;
use bevy::ecs::system::lifetimeless::Write;
use bevy::prelude::*;

#[derive(Component, Clone)]
pub enum TransformTween {
    Translation { start: Vec3, end: Vec3 },
    Rotation { axis: Dir3, start: f32, end: f32 },
}

impl Default for TransformTween {
    fn default() -> Self {
        Self::Translation {
            start: default(),
            end: default(),
        }
    }
}

impl TweenType for TransformTween {
    type Strategy = QueryDataTweenStrategy<Write<Transform>>;

    fn mutate_subject(
        &self,
        mut subject: <<Self::Strategy as TweenStrategy>::Param as TweenSystemParam>::Subject<
            '_,
            '_,
        >,
        factor: f32,
    ) {
        match self {
            TransformTween::Translation { start, end } => {
                subject.translation = start.lerp(*end, factor);
            }
            TransformTween::Rotation { axis, start, end } => {
                subject.rotation = Quat::from_axis_angle(axis.as_vec3(), start.lerp(*end, factor));
            }
        }
    }

    fn mutate_app(app: &mut App) {
        app.configure_sets(
            PostUpdate,
            TweenTypeSystems::<Self>::default().before(TransformSystems::Propagate),
        );
    }
}
