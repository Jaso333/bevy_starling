use std::time::Duration;

use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    feathers::{
        FeathersPlugins,
        controls::{FeathersListRow, FeathersSelect, list_rows_from_strings},
        dark_theme::create_dark_theme,
        display::caption,
        theme::{ThemeBackgroundColor, UiTheme},
        tokens,
    },
    prelude::*,
    reflect::enums::Enum,
    ui_widgets::ValueChange,
    window::WindowResized,
};
use bevy_starling::{
    color_material::*, core::*, sprite::*, text_color::TextColorTween, transform::*,
};

#[derive(Component, Clone, Default)]
struct ControlsUI;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(FeathersPlugins);
    app.add_plugins(TweenPlugin);
    app.add_plugins(TransformTween::plugin());
    app.add_plugins(SpriteTween::plugin());
    app.add_plugins(ColorMaterialTween::<MeshMaterial2d<ColorMaterial>>::plugin());
    app.add_plugins(TextColorTween::plugin());

    app.insert_resource(UiTheme(create_dark_theme()));

    app.add_systems(Startup, startup);
    app.add_systems(Update, update_viewport);

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

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        IsDefaultUiCamera,
        RenderLayers::none(),
    ));

    commands.spawn_scene(ui_root());
    commands.spawn_scene(sprite());
}

fn ui_root() -> impl Scene {
    bsn! {
        #ControlsUI
        ControlsUI
        Node {
            width: px(256.0),
            height: percent(100.0),
            padding: px(8.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            justify_self: JustifySelf::End,
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            @pattern_select()
            --
            @ease_function_select()
        ]
    }
}

fn pattern_select() -> impl Scene {
    let names = TweenPattern::default()
        .get_represented_enum_info()
        .map(|info| info.variant_names())
        .unwrap_or_else(|| &[]);

    let options = Box::new(
        names
            .iter()
            .map(|n| -> Box<dyn SceneList> {
                bsn! {
                    @FeathersListRow
                    Children [
                        @caption(*n)
                    ]
                }
                .into()
            })
            .collect::<Vec<_>>(),
    ) as Box<dyn SceneList>;

    bsn! {
        @FeathersSelect {
            @options: options
        }
        on(|_event: On<ValueChange<Entity>>| {
            println!("CHANGED");
        })
    }
}

fn ease_function_select() -> impl Scene {
    let names = EaseFunction::Linear
        .get_represented_enum_info()
        .map(|info| info.variant_names())
        .unwrap_or_else(|| &[]);

    bsn! {
        @FeathersSelect {
            @options: list_rows_from_strings(names, None)
        }
    }
}

fn sprite() -> impl Scene {
    bsn! {
        #Sprite
        Sprite {
            custom_size: { Some(Vec2::new(64.0, 64.0)) },
            color: { Color::linear_rgb(0.5, 1.0, 0.5) },
        }
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::QuadraticOut,
                cycles: TweenCycles::Infinite,
            }
            TweenMutator::<_>::from(TransformTranslationTween {
                start: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                end: Vec3 { x: 0.0, y: 256.0, z: 0.0 },
            })
        ]
    }
}

fn update_viewport(
    mut messages: MessageReader<WindowResized>,
    window_query: Query<&Window>,
    controls_query: Query<&Node, With<ControlsUI>>,
    mut camera_query: Query<&mut Camera, Without<IsDefaultUiCamera>>,
    scale: Res<UiScale>,
) {
    if messages.is_empty() {
        return;
    }

    messages.clear();

    let Ok(window) = window_query.single() else {
        return;
    };

    let Ok(node) = controls_query.single() else {
        return;
    };

    let Val::Px(ui_width) = node.width else {
        return;
    };

    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };

    let ui_width = ui_width * scale.0 * window.scale_factor();
    let width = window.physical_width() as f32 - ui_width;

    camera.viewport = Some(Viewport {
        physical_size: UVec2 {
            x: width as u32,
            y: window.physical_height(),
        },
        ..default()
    });
}

// fn sprite_0() -> impl Scene {
//     bsn! {
//         #Sprite0
//         Sprite {
//             custom_size: { Some(Vec2::new(64.0, 64.0)) },
//             color: { Color::linear_rgb(0.5, 1.0, 0.5) }
//         }
//         Tweens [
//             #Sprite0Tween
//             Tween {
//                 duration: Duration::from_secs_f32(2.0),
//                 function: EaseFunction::QuadraticInOut,

//             }
//             TweenMutator::<_>::from(TransformRotationTween {
//                 axis: Dir3::NEG_Z,
//                 start: 0.0,
//                 end: TAU,
//             })
//         ]
//     }
// }
