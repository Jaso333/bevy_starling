use std::f32::consts::TAU;
use std::time::Duration;

use bevy::prelude::*;
use bevy_tween::color_material::ColorMaterialColorTween;
use bevy_tween::core::*;
use bevy_tween::sprite::*;
use bevy_tween::transform::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(TweenPlugin);
    app.add_plugins(TransformTranslationTween::plugin());
    app.add_plugins(TransformRotationTween::plugin());
    app.add_plugins(SpriteColorTween::plugin());
    app.add_plugins(ColorMaterialColorTween::<MeshMaterial2d<ColorMaterial>>::plugin());

    app.add_systems(Startup, startup);
    app.add_systems(Update, update_active_tweens_text);
    app.add_systems(Update, launch_sprite);

    app.run();
}

#[derive(Component, Clone, Default)]
struct ActiveTweensText;

fn launched_sprite(name: String, x: f32) -> impl Scene {
    bsn! {
        Name::new(name)
        Sprite {
            custom_size: { Some(Vec2::new(32.0, 32.0)) },
            color: { Color::srgb(1.0, 1.0, 0.0) }
        }
        Tweens [
            #Launch
            TweenState {
                duration: Duration::from_secs_f32(2.0),
                function: EaseFunction::ExponentialOut,
                lifecycle: TweenLifecycle::Once
            }
            TransformTranslationTween {
                start: { Vec3::new(x, -256.0, 1.0) },
                end: { Vec3::new(x, 256.0, 1.0) }
            }
            on(|event: On<TweenFinished>, mut commands: Commands| {
                // despawn when the launch has finished
                commands.entity(event.tweened_entity).despawn();
            })
            --
            #Spin
            TweenState {
                duration: Duration::from_secs_f32(2.0),
                function: EaseFunction::ExponentialOut,
                lifecycle: TweenLifecycle::Once
            }
            TransformRotationTween {
                start: 0.0,
                end: { TAU * 3.0 } // 3 rotations
            }
        ]
    }
}

fn startup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    commands.spawn(Camera2d);

    commands.spawn_scene(bsn! {
        #SpinningSprite
        Sprite {
            custom_size: { Some(Vec2::new(128.0, 128.0)) },
            color: { Color::srgb(1.0, 0.0, 1.0) }
        }
        Tweens [
            #LeftRight
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
            #Rotation
            TweenState {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::Linear,
                lifecycle: TweenLifecycle::Repeat
            }
            TransformRotationTween {
                start: 0.0,
                end: TAU
            }
            --
            #Color
            TweenState {
                duration: Duration::from_secs_f32(2.0),
                function: EaseFunction::Linear,
                lifecycle: TweenLifecycle::BounceRepeat
            }
            SpriteColorTween {
                start: { Color::linear_rgb(1.0, 0.0, 0.0) },
                end: { Color::linear_rgb(0.0, 0.0, 1.0) }
            }
        ]
    });

    let mesh = meshes.add(Rectangle::new(128.0, 128.0));
    let material = materials.add(Color::srgb(1.0, 1.0, 1.0));

    commands.spawn_scene(bsn! {
        #GlowingMesh
        Mesh2d(mesh)
        MeshMaterial2d::<ColorMaterial>(material)
        Transform::from_xyz(0.0, 256.0, 0.0)
        Tweens [
            #Color
            TweenState {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::Linear,
                lifecycle: TweenLifecycle::BounceRepeat
            }
            ColorMaterialColorTween::<MeshMaterial2d<ColorMaterial>> {
                start: Color::linear_rgb(0.0, 1.0, 0.0),
                end: Color::linear_rgb(1.0, 1.0, 1.0),
            }
        ]
    });

    commands.spawn_scene(bsn! {
        ActiveTweensText
        Text
    });
}

fn launch_sprite(
    mut index: Local<u32>,
    input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    let name = format!("Launched Sprite {}", *index);
    commands.spawn_scene(launched_sprite(name, rand::random_range(-256.0..256.0)));

    *index += 1;
}

fn update_active_tweens_text(
    mut text_query: Query<&mut Text, With<ActiveTweensText>>,
    target_query: Query<(&Name, &Tweens)>,
    tween_query: Query<&Name>,
) {
    for mut text in text_query.iter_mut() {
        text.0.clear();
        text.0.push_str("Active Tweens:\n");

        for (target_name, tweens) in target_query.iter() {
            text.0.push_str("\n\n");
            text.0.push_str(target_name);
            text.0.push_str(":");

            for tween_entity in tweens.iter() {
                let Ok(name) = tween_query.get(tween_entity) else {
                    continue;
                };

                text.0.push_str("\n    ");
                text.0.push_str(name);
            }
        }
    }
}
