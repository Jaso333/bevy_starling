use std::f32::consts::TAU;
use std::time::Duration;

use bevy::prelude::*;
use bevy_starling::color_material::*;
use bevy_starling::core::*;
use bevy_starling::sprite::*;
use bevy_starling::transform::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(TweenPlugin);
    app.add_plugins(TransformTween::plugin());
    app.add_plugins(SpriteTween::plugin());
    app.add_plugins(ColorMaterialTween::<MeshMaterial2d<ColorMaterial>>::plugin());

    app.add_systems(Startup, startup);
    app.add_systems(
        Update,
        (
            update_entities_being_tweened_text,
            update_active_tweens_text,
        ),
    );
    app.add_systems(Update, launch_sprite);

    app.run();
}

#[derive(Component, Clone, Default)]
struct EntitiesBeingTweenedText;

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
            Tween {
                duration: Duration::from_secs_f32(2.0),
                function: EaseFunction::ExponentialOut,
                lifecycle: TweenLifecycle::OnceTransient
            }
            TweenMutator::<_>::from(TransformTranslationTween {
                start: Vec3::new(x, -256.0, 1.0),
                end: Vec3::new(x, 256.0, 1.0),
            })
            on(|event: On<TweenFinished>, mut commands: Commands| {
                // despawn when the launch has finished
                commands.entity(event.tweened_entity).despawn();
            })
            --
            #Spin
            Tween {
                duration: Duration::from_secs_f32(2.0),
                function: EaseFunction::ExponentialOut,
                lifecycle: TweenLifecycle::OnceTransient
            }
            TweenMutator::<_>::from(TransformRotationTween {
                axis: Dir3::NEG_Z,
                start: 0.0,
                end: TAU * 3.0,
            })
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
            Tween {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::QuadraticInOut,
                lifecycle: TweenLifecycle::PingPong
            }
            TweenMutator::<_>::from(TransformTranslationTween {
                start: Vec3::new(-256.0, 0.0, 0.0),
                end: Vec3::new(256.0, 0.0, 0.0)
            })
            --
            #Rotation
            Tween {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::Linear,
                lifecycle: TweenLifecycle::Repeat
            }
            TweenMutator::<_>::from(TransformRotationTween {
                axis: Dir3::NEG_Z,
                start: 0.0,
                end: TAU
            })
            --
            #Color
            Tween {
                duration: Duration::from_secs_f32(2.0),
                function: EaseFunction::Linear,
                lifecycle: TweenLifecycle::PingPong
            }
            TweenMutator::<_>::from(SpriteColorTween {
                start: Color::linear_rgb(1.0, 0.0, 0.0),
                end: Color::linear_rgb(0.0, 0.0, 1.0)
            })
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
            Tween {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::Linear,
                lifecycle: TweenLifecycle::PingPong
            }
            TweenMutator::<_>::from(ColorMaterialColorTween {
                start: Color::linear_rgb(0.0, 1.0, 0.0),
                end: Color::linear_rgb(1.0, 1.0, 1.0)
            })
        ]
    });

    commands.spawn_scene(bsn! {
        #PickableSprite
        Sprite {
            custom_size: { Some(Vec2::new(64.0, 64.0)) },
            color: { Color::linear_rgb(1.0, 0.5, 0.0) }
        }
        Transform::from_xyz(-128.0, -128.0, 0.0)
        Pickable
        Tweens [
            #HoverTween
            Tween {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::Linear,
                lifecycle: TweenLifecycle::OnceRetained,
                direction: TweenDirection::Backwards,
                state: TweenState::Finished,
                progress: 1.0
            }
            TweenMutator::<_>::from(SpriteColorTween {
                start: Color::linear_rgb(1.0, 0.5, 0.0),
                end: Color::linear_rgb(0.0, 0.5, 1.0)
            })
            TweenMutator::<_>::from(TransformRotationTween {
                axis: Dir3::NEG_Z,
                start: 0.0,
                end: TAU
            })
        ]
        on(on_pickable_sprite_over)
        on(on_pickable_sprite_out)
    });

    let font_size = FontSize::Px(12.0);

    commands.spawn_scene(bsn! {
        EntitiesBeingTweenedText
        Text
        TextFont {
            font_size: { font_size.clone() }
        }
    });

    commands.spawn_scene(bsn! {
        ActiveTweensText
        Text
        Node {
            justify_self: JustifySelf::End,
            width: px(450.0)
        }
        TextLayout::default().with_justify(Justify::End)
        TextFont {
            font_size: { font_size.clone() }
        }
    });
}

fn on_pickable_sprite_over(
    event: On<PointerOver>,
    sprite_query: Query<&Tweens>,
    mut tween_query: Query<&mut Tween>,
) {
    let tween_entity = sprite_query
        .get(event.entity)
        .unwrap()
        .iter()
        .next()
        .unwrap();

    let mut tween = tween_query.get_mut(tween_entity).unwrap();
    tween.bounce();
    tween.state = TweenState::Running; // in case it finished
}

fn on_pickable_sprite_out(
    event: On<PointerOut>,
    sprite_query: Query<&Tweens>,
    mut tween_query: Query<&mut Tween>,
) {
    let tween_entity = sprite_query
        .get(event.entity)
        .unwrap()
        .iter()
        .next()
        .unwrap();

    let mut tween = tween_query.get_mut(tween_entity).unwrap();
    tween.bounce();
    tween.state = TweenState::Running; // in case it finished
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

fn update_entities_being_tweened_text(
    mut text_query: Query<&mut Text, With<EntitiesBeingTweenedText>>,
    target_query: Query<(&Name, &Tweens)>,
    tween_query: Query<&Name>,
) {
    for mut text in text_query.iter_mut() {
        text.0.clear();
        text.0.push_str("Tweened Entities:\n");

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

fn update_active_tweens_text(
    mut text_query: Query<&mut Text, With<ActiveTweensText>>,
    tween_query: Query<(&Name, &Tween)>,
) {
    for mut text in text_query.iter_mut() {
        text.0.clear();
        text.0.push_str("Active Tweens:\n");

        for (name, tween) in tween_query.iter() {
            text.0.push_str("\n\n");
            text.0.push_str(name);
            text.0.push_str(":");

            text.0.push_str(&format!(
                "\nProgress: {}    ",
                format!("{:.2}", tween.progress)
            ));
            text.0
                .push_str(&format!("\nDirection: {:?}    ", tween.direction));
            text.0.push_str(&format!("\nState: {:?}    ", tween.state));
        }
    }
}
