use bevy::prelude::*;
use bevy_tween::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(TweenPlugin);
    app.add_plugins(TransformTranslationTween::plugin());
    app.add_plugins(MeshMaterial3dStandardMaterialBaseColorTween::plugin());
}
