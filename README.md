**bevy_starling** is a tweening library for the [Bevy game engine](https://bevyengine.org/).

# Design

Whilst other tweening libraries are available for Bevy, this one aims to leverage Rust's type system to create safe, flexible, and efficient tweening solutions.

Tweens are created as their own entities with a relationship (`TweenOf`) to the entity that they mutate. Therefore, tweens in this library are always targeted at entities.

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



# License

bevy_starling is free and open source. All code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.