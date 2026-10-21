**bevy_starling** is a tweening library for the [Bevy game engine](https://bevyengine.org/).

# Design

Whilst other tweening libraries are available for Bevy, this one aims to leverage Rust's type system to create safe, flexible, and efficient tweening solutions.

Tweens are created as their own entities with a relationship (`TweenOf`) to the entity that they mutate. Therefore, tweens in this library are always targeted at entities.

All components are BSN-friendly (see usage section).

The following traits/structs make up the core of the library:

- `TweenType`
  - Implemented by tween types. Typically there is one type per component being tweened, but this is not mandated. This does not represent the mutation part of the tween, just the retrieval of the subject.
- `TweenPlugin`
  - Adds general tweening functionality to the app.
- `TweenTypePlugin`
  - Adds systems for a particular tween type to the app. This is accessible by `TweenType::plugin` and does not need to be used directly.
- `TweenSystemParam`
  - Implemented by system parameters that retreive the subjects of the tween. Implementations are used by `TweenType` to form the implementation of it.
- `TweenSubject`
  - Wraps the lifetime-bounded type of a tween subject. A tween subject is the data related to an entity that is the subject of a tween. For example, this would be a mutable refernece to the `Transform` component. Instances of subjects are produced by `TweenSystemParam`s.
- `Tween`
  - The general tween state component. This contains things like duration, number of cycles, progress, etc. This is not tied to a particular tween type.
- `TweenMutators`
  - A component that contains the mutators for a single tween. It should be paired with `Tween` to provide the data needed for all the tween systems. This component is parameterized by `TweenType`, which indirectly associates it with a particular subject type that the contained mutators must be compatible with.
- `TweenMutator`
  - Implemented by structs that contain specific start/end values for the actual application of the tween progress. These are associated with a specific type of subject, and they can be added to a `TweenMutators` component as long as the subject type matches the subject type of the associated `TweenType`.

The library contains common `TweenSystemParam` implementations:

- `QueryTweenParam`
  - The most common case: allows a tween type to specify a `QueryData` needed for the tween subject. For example, the `TransformTweenType` uses `QueryTweenParam<Write<Transform>>`.
- `AssetTweenParam`
  - This is used to tween assets that are referenced by a component on the entity. The component must implement `Deref<Target = Handle<A>>` in order for it to be compatible. Most Bevy new-type style components that hold a handle to an asset implement `Deref`, e.g. `MeshMaterial2d`.

The library also contains common `TweenType`s for use:

- `TransformTween`
  - Adds tweens for the `Transform` component.
  - Mutators exist for translation and rotation.
  - Add crate feature `transform` to use.
- `SpriteTween`
  - Adds tweens for the `Sprite` component.
  - Mutator exists for the color field.
  - Add crate feature `sprite` to use.
- `TextColorTween`
  - Adds tweens for the `TextColor` component.
  - This is also the mutator, because it is a new-type component.
  - Add crate feature `text_color` to use.
- `ColorMaterialTween`
  - Adds tweens for `ColorMaterial` assets referenced by the following components:
    - `MeshMaterial2d`
  - Mutator exists for the color field.
  - Add crate feature `color_material` to use.

# Usage

Add the crate to your project as follows:

```toml
bevy_starling = { version = "0.1.0", features = ["full"] }
```

By default, no built-in tweens will be added unless the corresponding feature is enabled (see above). Using the `full` feature will enable all built-in tween types.

Then add the plugins:

```rust
app.add_plugins(TweenPlugins);
```

## Spawning

Provided you have the appropriate features for the tween types and mutators, the following will add a tween to an existing entity:

```rust
commands
    .entity(entity)
    .with_related::<TweenOf>((
        Tween {
            duration: Duration::from_secs_f32(1.0),
            function: EaseFunction::QuadraticInOut,
            pattern: TweenPattern::PingPong,
            ..default()
        },
        TweenMutators::<TransformTween>::from(TransformTranslationTween {
            start: Vec3::ZERO,
            end: Vec3::new(0.0, 128.0, 0.0),
        })
        .with(TransformRotationTween {
            axis: Dir3::NEG_Z,
            start: 0.0,
            end: TAU,
        }),
    ));
```

In the above example, a translation and rotation mutator is used in the tween that is added to the entity.

It's also just as easy to create BSN scenes with tweens on them. The equivalent of the above would be:

```rust
fn my_sprite() -> impl Scene {
    bsn! {
        Sprite {
            custom_size: { Some(Vec2::splat(64.0)) },
            color: Color::srgb(0.75, 1.0, 0.25),
        }
        Tweens [
            Tween {
                duration: Duration::from_secs_f32(1.0),
                function: EaseFunction::QuadraticInOut,
                pattern: TweenPattern::PingPong,
            }
            TweenMutators::<TransformTween>::from(TransformTranslationTween {
                start: Vec3::ZERO,
                end: Vec3::new(0.0, 128.0, 0.0),
            })
            .with(TransformRotationTween {
                axis: Dir3::NEG_Z,
                start: 0.0,
                end: TAU,
            })
        ]
    }
}
```

The `TweenFinished` event is triggered for tweens that can exhaust their cycles:

```rust
fn my_sprite() -> impl Scene {
    bsn! {
        ...
        on(|event: On<TweenFinished>| {
            println!("Tween of entity {} finished!", event.tweened_entity);
        })
    }
}
```

## Custom Mutators

If you wish to mutate a component in a unique way, or in a way that is not included in the built-in mutators, simply implement it:

```rust
#[derive(Clone, Default)]
pub struct SpriteColorHueTween {
    pub start: f32,
    pub end: f32,
}

impl TweenMutator for SpriteColorHueTween {
    type Subject = QueryTweenSubject<Write<Sprite>>;

    fn mutate(&self, subject: &mut Mut<'_, Sprite>, factor: f32) {
        subject.color.set_hue(self.start.lerp(self.end, factor));
    }
}
```

## Custom Tween Types

If you have your own component you want to tween, or there is a Bevy component that is not covered by the built-in tween types, you can create your own:

```rust
#[derive(Component, Clone, Default)]
struct Foo {
    value: f32,
}

struct FooTween;

impl TweenType for FooTween {
    type Param = QueryTweenParam<'static, 'static, &'static mut Foo>;
}

#[derive(Clone, Default)]
struct FooValueTween {
    start: f32,
    end: f32,
}

impl TweenMutator for FooValueTween {
    type Subject = QueryTweenSubject<&'static mut Foo>;

    fn mutate(&self, subject: &mut Mut<'_, Foo>, factor: f32) {
        subject.value = self.start.lerp(self.end, factor);
    }
}
```

When creating custom tween types, it is necesary to add the plugin to your app:

```rust
app.add_plugins(FooTween::plugin());
```

## Custom Tween System Param

This is a more complex topic that is out of the scope of this readme; it is also rarely needed. Please refer to examples such as `QueryTweenParam` and `AssetTweenParam` for guidance on how to implement your own.

# License

bevy_starling is free and open source. All code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.