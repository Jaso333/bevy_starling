use std::f32::consts::TAU;
use std::time::Duration;

use bevy::prelude::*;
use bevy_tween::core::*;
use bevy_tween::sprite::*;
use bevy_tween::transform::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(TweenPlugin);
    app.add_plugins(TransformTranslationTween::plugin());
    app.add_plugins(TransformRotationZTween::plugin());
    app.add_plugins(SpriteColorTween::plugin());

    app.add_systems(Startup, startup);

    app.run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn_scene(bsn! {
        Sprite {
            custom_size: { Some(Vec2::new(128.0, 128.0)) },
            color: { Color::srgb(1.0, 0.0, 1.0) }
        }
        Tweens [
            TweenState {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::QuadraticInOut,
                lifecycle: TweenLifecycle::BounceRepeat
            }
            TransformTranslationTween {
                start: { Vec3::new(-256.0, 0.0, 0.0) },
                end: { Vec3::new(256.0, 0.0, 0.0) }
            }
            --
            TweenState {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::Linear,
                lifecycle: TweenLifecycle::Repeat
            }
            TransformRotationZTween {
                start: 0.0,
                end: TAU
            }
            --
            TweenState {
                duration: Duration::from_secs_f32(2.0),
                function: EaseFunction::Linear,
                lifecycle: TweenLifecycle::BounceRepeat
            }
            SpriteColorTween {
                start: { Color::srgb(1.0, 0.0, 0.0) },
                end: { Color::srgb(0.0, 0.0, 1.0) }
            }
        ]
    });
}
