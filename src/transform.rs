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
    type Strategy = QueryDataTweenMutator<Write<Transform>>;

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
