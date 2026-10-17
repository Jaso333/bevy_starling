use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy_starling::color_material::*;
use bevy_starling::core::*;
use bevy_starling::sprite::*;
use bevy_starling::text_color::TextColorTween;
use bevy_starling::transform::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(TweenPlugin);
    app.add_plugins(TransformTween::plugin());
    app.add_plugins(SpriteTween::plugin());
    app.add_plugins(ColorMaterialTween::<MeshMaterial2d<ColorMaterial>>::plugin());
    app.add_plugins(TextColorTween::plugin());

    app.add_systems(Startup, startup);

    app.run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn_scene(sprite());
}

fn sprite() -> impl Scene {
    bsn! {
        #Sprite
        Sprite {
            custom_size: { Some(Vec2::new(64.0, 64.0)) },
            color: { Color::linear_rgb(0.5, 1.0, 0.5) }
        }
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::QuadraticOut,
                pattern: TweenPattern::PingPong,
                cycles: TweenCycles::Infinite
            }
            TweenMutator::<_>::from(TransformRotationTween {
                axis: Dir3::NEG_Z,
                start: 0.0,
                end: PI
            })
        ]
    }
}
