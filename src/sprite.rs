use crate::core::*;
use crate::query_data::*;
use bevy::ecs::system::lifetimeless::Write;
use bevy::prelude::*;

#[derive(Component, Clone)]
pub enum SpriteTween {
    Color { start: Color, end: Color },
}

impl Default for SpriteTween {
    fn default() -> Self {
        Self::Color {
            start: default(),
            end: default(),
        }
    }
}

impl TweenType for SpriteTween {
    type Strategy = QueryDataTweenStrategy<Write<Sprite>>;

    fn mutate_subject(
        &self,
        mut subject: <<Self::Strategy as TweenStrategy>::Param as TweenSystemParam>::Subject<
            '_,
            '_,
        >,
        factor: f32,
    ) {
        match self {
            SpriteTween::Color { start, end } => {
                subject.color = start.mix(end, factor);
            }
        }
    }
}
