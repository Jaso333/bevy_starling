use std::{f32::consts::TAU, time::Duration};

use bevy::prelude::*;
use bevy_starling::{TweenPlugins, core::*, sprite::*, transform::*};

const DURATION: f32 = 2.0;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(TweenPlugins);

    app.add_systems(Startup, startup);

    app.run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
    ));

    commands.spawn_scene_list(sprites_row_1());
    commands.spawn_scene_list(sprites_row_2());
}

fn sprites_row_1() -> impl SceneList {
    bsn_list! {
        @sprite()
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(DURATION),
                function: EaseFunction::QuadraticOut,
                cycles: TweenCycles::Infinite,
            }
            TweenMutators::<TransformTween>::from(TransformTranslationTween {
                start: Vec3::new(-256.0, 128.0, 0.0),
                end: Vec3::new(-256.0, 256.0, 0.0),
            })
        ]
        --
        @sprite()
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(DURATION),
                function: EaseFunction::QuadraticOut,
                cycles: TweenCycles::Infinite,
                pattern: TweenPattern::PingPong,
            }
            TweenMutators::<TransformTween>::from(TransformTranslationTween {
                start: Vec3::new(-128.0, 128.0, 0.0),
                end: Vec3::new(-128.0, 256.0, 0.0),
            })
        ]
        --
        @sprite()
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(DURATION),
                function: EaseFunction::QuadraticOut,
                cycles: TweenCycles::Infinite,
                pattern: TweenPattern::ForwardReverse,
            }
            TweenMutators::<TransformTween>::from(TransformTranslationTween {
                start: Vec3::new(0.0, 128.0, 0.0),
                end: Vec3::new(0.0, 256.0, 0.0),
            })
        ]
        --
        @sprite()
        Transform::from_xyz(128.0, 128.0, 0.0)
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(DURATION),
                function: EaseFunction::QuadraticOut,
                cycles: TweenCycles::Infinite,
                pattern: TweenPattern::PingPong,
            }
            TweenMutators::<TransformTween>::from(TransformRotationTween {
                axis: Dir3::NEG_Z,
                start: 0.0,
                end: TAU,
            })
        ]
        --
        @sprite()
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(DURATION),
                function: EaseFunction::QuadraticOut,
                cycles: TweenCycles::Infinite,
                pattern: TweenPattern::ForwardReverse,
            }
            TweenMutators::<TransformTween>::from(
                TransformRotationTween {
                    axis: Dir3::NEG_Z,
                    start: 0.0,
                    end: TAU,
                }
            ).with(
                TransformTranslationTween {
                    start: Vec3::new(256.0, 128.0, 0.0),
                    end: Vec3::new(256.0, 256.0, 0.0),
                }
            )
        ]
    }
}

fn sprites_row_2() -> impl SceneList {
    bsn_list! {
        @sprite()
        Transform::from_xyz(-128.0, 0.0, 0.0)
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(DURATION),
                function: EaseFunction::QuadraticOut,
                cycles: TweenCycles::Infinite,
                pattern: TweenPattern::PingPong,
            }
            TweenMutators::<SpriteTween>::from(SpriteColorTween {
                start: Color::srgba(1.0, 0.25, 0.75, 1.0),
                end: Color::srgba(1.0, 0.25, 0.75, 0.0),
            })
        ]
        --
        @sprite()
        Transform::from_xyz(0.0, 0.0, 0.0)
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(DURATION),
                function: EaseFunction::QuadraticOut,
                cycles: TweenCycles::Infinite,
            }
            TweenMutators::<SpriteTween>::from(SpriteColorTween {
                start: Color::srgb(1.0, 0.25, 0.75),
                end: Color::srgb(0.25, 0.75, 1.0),
            })
        ]
        --
        @sprite()
        Transform::from_xyz(128.0, 0.0, 0.0)
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(DURATION),
                function: EaseFunction::QuadraticOut,
                cycles: TweenCycles::Infinite,
                pattern: TweenPattern::ForwardReverse,
            }
            TweenMutators::<SpriteTween>::from(SpriteColorTween {
                start: Color::hsv(0.0, 1.0, 1.0),
                end: Color::hsv(180.0, 1.0, 1.0),
            })
        ]
    }
}

fn sprite() -> impl Scene {
    bsn! {
        Sprite {
            custom_size: { Some(Vec2::splat(64.0)) },
            color: Color::srgb(0.75, 1.0, 0.25),
        }
    }
}
