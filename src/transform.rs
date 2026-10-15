use crate::core::*;
use crate::query_data::*;
use bevy::ecs::system::lifetimeless::Write;
use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct TransformTranslationTween {
    pub start: Vec3,
    pub end: Vec3,
}

impl TweenType for TransformTranslationTween {
    type Strategy = QueryDataTweenStrategy<Write<Transform>>;

    fn mutate_subject(
        &self,
        mut item: <<Self::Strategy as TweenStrategy>::Param as TweenSystemParam>::Subject<'_, '_>,
        factor: f32,
    ) {
        item.translation = self.start.lerp(self.end, factor);
    }

    fn mutate_app(app: &mut App) {
        app.configure_sets(
            PostUpdate,
            TweenTypeSystems::<Self>::default().before(TransformSystems::Propagate),
        );
    }
}

#[derive(Component, Clone)]
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

impl TweenType for TransformRotationTween {
    type Strategy = QueryDataTweenStrategy<Write<Transform>>;

    fn mutate_subject(
        &self,
        mut item: <<Self::Strategy as TweenStrategy>::Param as TweenSystemParam>::Subject<'_, '_>,
        factor: f32,
    ) {
        item.rotation =
            Quat::from_axis_angle(self.axis.as_vec3(), self.start.lerp(self.end, factor));
    }

    fn mutate_app(app: &mut App) {
        app.configure_sets(
            PostUpdate,
            TweenTypeSystems::<Self>::default().before(TransformSystems::Propagate),
        );
    }
}
