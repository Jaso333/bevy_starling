use std::time::Duration;

use bevy::prelude::*;
use bevy_tween::core::*;
use bevy_tween::transform::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(TweenPlugin);
    app.add_plugins(TransformTranslationTween::plugin());

    app.add_systems(Startup, startup);

    app.run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn_scene(bsn! {
        Sprite {
            custom_size: { Some(Vec2::new(32.0, 32.0)) },
            color: { Color::srgb(1.0, 0.0, 1.0) }
        }
        Tweens [
            @my_tween(Vec2::new(-128.0, 0.0), Vec2::new(128.0, 0.0))
        ]
    });
}

fn my_tween(start: Vec2, end: Vec2) -> impl Scene {
    bsn! {
        TweenState::new(Duration::from_secs_f32(3.0), EaseFunction::QuadraticInOut)
        TransformTranslationTween {
            start: { start.extend(0.0) },
            end: { end.extend(0.0) }
        }
        on(on_tween_finished)
    }
}

fn on_tween_finished(
    event: On<TweenFinished>,
    sprite_query: Query<(Entity, &Transform)>,
    mut commands: Commands,
) {
    let Ok((entity, transform)) = sprite_query.get(event.tweened_entity) else {
        return;
    };

    let end = match transform.translation.x > 0.0 {
        true => -1.0,
        false => 1.0,
    } * 128.0;

    commands.spawn_scene(bsn! {
        @my_tween(Vec2::new(transform.translation.x, 0.0), Vec2::new(end, 0.0))
        TweenOf(entity)
    });
}
