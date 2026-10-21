use std::{
    any::TypeId,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::Arc,
    time::Duration,
};

use bevy_app::prelude::*;
use bevy_curve::prelude::*;
use bevy_ecs::{
    prelude::*,
    system::{StaticSystemParam, SystemParam, SystemParamItem},
};
use bevy_reflect::Reflect;
use bevy_time::prelude::*;

/// Introduces general-purpose tweening functionality to the app. This also adds specific tween types to the app depending on chosen features.
///
/// Unlike other crates of a similar nature, this one uses separate entities with relationships to the entities being tweened.
/// The entity containing the the state of the tween is a [`TweenOf`] the entity containing the component that is tweened.
///
/// Tweens require two components: [`Tween`] and [`TweenMutators`]. The former being the state and behaviour, the latter being the part that associates with a particular type of tween.
///
/// [`TweenMutators`] contains any number of compatible [`TweenMutator`]s.
/// If a [`TweenMutator::Subject`] associates with the same [`TweenSubject`] as the [`TweenType::Param`] associated with the [`TweenType`] of the [`TweenMutators`], then it is compatible.
/// Different [`TweenType`]s can be implemented for the same [`TweenSubject`] types, meaning that it is possible that mutators can be reused for different tween types.
///
/// When [`TweenType`] is implemented, a [`TweenSystemParam`] type is associated. This defines how the data is provided to the mutator via a [`TweenSubject`] from within a system.
///
/// Tween processing is broken into three steps:
///
/// - [`PreUpdate`]
///   - [`TweenSystems::Finish`]
/// - [`PostUpdate`]
///   - [`TweenSystems::Progress`]
///   - [`TweenSystems::Mutate`]
///
/// When a tween finishes, the [`TweenFinished`] event is triggered on the tween entity during the [`TweenSystems::Finish`] set execution.
///
/// The tween begins in the [`TweenState::Init`] state by default. This means that the first time the tween is progressed, the [`Tween::progress`] value is set to [`0.0`].
/// This behaviour is the default because the current frame's delta does not apply to a tween that did not exist in the previous frame.
/// In order to have the first progression of the tween be affected by the current frame delta, spawn a tween with the [`Tween::state`] set to [`TweenState::Running`] instead.
///
/// [`TweenType`] in typically implemented per component.
/// However, if the retrieval of the [`TweenSubject`] is more exotic, a [`TweenType`] that demands a special [`TweenSystemParam`] can be implemented.
///
/// [`TweenSystemParam`] is what parameterizes the system used to mutate [`TweenSubject`]s. Through the power of Bevy's [`SystemParam`], the possibility of tweens is effectively unbounded.
#[derive(Default)]
pub struct TweenPlugin;

impl Plugin for TweenPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            (TweenSystems::Progress, TweenSystems::Mutate).chain(),
        );

        app.add_systems(PreUpdate, finish_tweens.in_set(TweenSystems::Finish));
        app.add_systems(PostUpdate, progress_tweens.in_set(TweenSystems::Progress));
    }
}

/// The plugin type that added specific tween types to the app.
///
/// This does not need to be used directly, [`GetTweenTypePlugin::plugin`] will return the plugin for a given [`TweenType`].
pub struct TweenTypePlugin<T: TweenType> {
    _phantom: PhantomData<T>,
}

impl<T: TweenType> Default for TweenTypePlugin<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: TweenType + Send + Sync + 'static> Plugin for TweenTypePlugin<T> {
    fn build(&self, app: &mut App) {
        // some tween types need to perform app configuration (e.g. transform tweens)
        T::configure_app(app);

        app.configure_sets(
            PostUpdate,
            TweenTypeSystems::<T>::default().in_set(TweenSystems::Mutate),
        );

        app.add_systems(
            PostUpdate,
            mutate_subjects::<T>.in_set(TweenTypeSystems::<T>::default()),
        );
    }
}

/// Used to blanket-implement a convenience function for getting the plugin for a given tween type.
pub trait GetTweenTypePlugin<T: TweenType> {
    /// Gets the plugin for this tween type.
    ///
    /// Use this function to get and add your custom tween type plugin to the app.
    fn plugin() -> TweenTypePlugin<T> {
        TweenTypePlugin::<T>::default()
    }
}

impl<T: TweenType> GetTweenTypePlugin<T> for T {}

/// The categorisation of systems in the plugins.
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum TweenSystems {
    Progress,
    Mutate,
    Finish,
}

/// The generic system set used for each tween type.
///
/// This allows tween types to organise their system, or allow other systems to organise themselves around it.
pub struct TweenTypeSystems<T: TweenType> {
    _marker: PhantomData<T>,
}

impl<T: TweenType> Default for TweenTypeSystems<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: TweenType + Send + Sync + 'static> SystemSet for TweenTypeSystems<T> {
    fn dyn_clone(&self) -> Box<dyn SystemSet> {
        Box::new(Self::default())
    }
}

impl<T: TweenType> Debug for TweenTypeSystems<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TweenTypeSystems")
            .field("_marker", &self._marker)
            .finish()
    }
}

impl<T: TweenType + 'static> Hash for TweenTypeSystems<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        TypeId::of::<T>().hash(state);
    }
}

impl<T: TweenType> PartialEq for TweenTypeSystems<T> {
    fn eq(&self, _other: &Self) -> bool {
        // zero-sized, always true
        true
    }
}

impl<T: TweenType> Eq for TweenTypeSystems<T> {}

/// The trait to be implemented for a particular tween type.
///
/// A tween type has its own system added to the app (provided the plugin is added) for mutation of the subjects.
pub trait TweenType {
    /// The parameter that is used in the system to produce the subject required for the tween type.
    type Param: TweenSystemParam;

    /// Allows the tween type to mutate the app when its plugin is added.
    fn configure_app(#[allow(unused)] app: &mut App) {}
}

/// Lifetime-erased trait that carries the lifetime-bounded item type produced by a [`TweenSystemParam`] and mutated by a [`TweenMutator`].
pub trait TweenSubject {
    /// The item with world and system lifetime parameters.
    type Item<'w, 's>;
}

/// Implemented by mutator types; the types that typically have the start and end points of the tween, e.g. "TransformTranslationTween".
///
/// The implementor describes the subject of the tween, which is fed into the function for performing the mutation.
pub trait TweenMutator {
    /// The subject that is produced by whatever [`TweenType`] is chosen when adding to the world.
    type Subject: TweenSubject;

    /// Performs the mutation of the subject.
    fn mutate(&self, subject: &mut <Self::Subject as TweenSubject>::Item<'_, '_>, factor: f32);
}

/// Implemented by system parameters that retrieve subjects for the mutators.
///
/// Its rare to need to implement this unless you have exotic or game-specific needs.
/// This crate contains a system param type that generic over any kind of QueryData, and this will be used in most cases.
pub trait TweenSystemParam: SystemParam {
    /// The type of the subject that is retrieved from the system param.
    type Subject: TweenSubject + 'static;

    /// Gets the subject from the system parameter.
    fn get_subject_mut<'a, 'w, 's>(
        item: &'a mut SystemParamItem<'w, 's, Self>,
        entity: Entity,
    ) -> Option<<Self::Subject as TweenSubject>::Item<'a, 's>>;
}

/// Triggered when a tween finishes.
#[derive(EntityEvent)]
pub struct TweenFinished {
    /// The entity responsible for the tween.
    #[event_target]
    pub tween_entity: Entity,
    /// The entity that has been the subject of the tween.
    pub tweened_entity: Entity,
}

/// Defines the lifetime of a tween. In most cases, you want the tween entity to despawn when complete.
///
/// However, it is possible to retain it for further playback at a later time.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TweenLifetime {
    /// The tween entity will despawn once it is finished. This is not effective if [`TweenCycles::Infinite`] is used.
    #[default]
    Transient,
    /// The tween entity will be retained after it is finished.
    Retained,
}

/// The pattern that the tween follows during its progress from start to finish.
///
/// Be aware that "two-leg" patterns (ping-pong, forward-reverse) account for only one "cycle",
/// so they may look twice as fast, but the duration of a tween defines the duration of one cycle of a pattern.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Reflect)]
pub enum TweenPattern {
    /// Tween from A to B.
    #[default]
    Single,
    /// Tween from A to B to A, with the return journey using a reverse of the function in the tween.
    ///
    /// Note that this may be different from the definition of "ping-pong" used in other animation systems.
    /// Considering an analogy where a ping-pong ball is hit back and forth; the ball moves with a ease-out curve.
    /// When the ball is hit, it starts from the beginning of that easing curve, regardless of which player hits it.
    PingPong,
    /// Tween from A to B to A, with the return journey retracing the original journey in reverse.
    ForwardReverse,
}

/// Defines how many cycles the tween should play back before finishing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TweenCycles {
    /// Typically, this variant would be used with `1` to play the tween's pattern once.
    /// When a cycle completes, this number is decremented.
    ///
    /// [`TweenFinished`] events do not trigger until the number hits 0.
    Finite(u32),
    /// Repeat the tween indefinitely.
    ///
    /// [`TweenFinished`] events never trigger when using this variant.
    Infinite,
}

impl TweenCycles {
    /// Convenience for the typical scenario of running a tween once.
    pub fn once() -> Self {
        Self::Finite(1)
    }

    /// Returns if the tween has no cycles left to run.
    pub fn is_exhausted(&self) -> bool {
        match self {
            TweenCycles::Finite(count) if *count == 0 => true,
            _ => false,
        }
    }

    /// Decrements the cycle count by 1. This has no effect on infinite cycles.
    pub fn decrement(&mut self) {
        match self {
            TweenCycles::Finite(cycles) => *cycles -= 1,
            TweenCycles::Infinite => (),
        }
    }
}

impl Default for TweenCycles {
    fn default() -> Self {
        Self::once()
    }
}

/// The direction of the tween's progression.
///
/// [`Tween::progress`] always travels from 0 to 1, but this inverts the value when passed to the mutators.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TweenDirection {
    /// The standard direction.
    #[default]
    Forward,
    /// Inverts the progress when provided to the mutators.
    Backward,
}

impl TweenDirection {
    /// Gets the opposite direction.
    fn to_opposite(&self) -> Self {
        match self {
            TweenDirection::Forward => Self::Backward,
            TweenDirection::Backward => Self::Forward,
        }
    }
}

/// The high-level state of the tween.
///
/// This allows the systems to differentiate between a tween that has just spawned, and one that is running.
///
/// There is no "finished" state because the "finished" state is implied by [`TweenCycles::is_exhausted`].
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum TweenState {
    /// The default initial state of a tween. This ensures that the first mutation represents a progress of 0.0.
    /// This is justified by the fact that the tween did not exist in the previous frame, so the delta time since the last frame should not be applied.
    /// This default behaviour can be avoided by simply spawning with the "Running" state instead.
    #[default]
    Init,
    /// Once "Init" is encountered by the progression system, it is switched to this state for the next frame.
    /// When in this state, delta time since the previous frame is applied to the [`Tween::progress`] according to the tween duration.
    Running,
}

/// Contains the tween data independent of the tween type.
#[derive(Component, Clone, Debug)]
pub struct Tween {
    /// How long each tween cycle should last.
    pub duration: Duration,
    /// The easing function to use.
    pub function: EaseFunction,
    /// The lifetime of the tween entity.
    pub lifetime: TweenLifetime,
    /// The pattern of each individual cycle.
    pub pattern: TweenPattern,
    /// The number of cycles to perform before finishing.
    pub cycles: TweenCycles,
    /// The general direction of the tween.
    pub direction: TweenDirection,
    /// The overall state of the tween.
    pub state: TweenState,
    /// The progress of the tween, normalised to between 0 and 1. This is updated internally.
    pub progress: f32,
}

impl Default for Tween {
    fn default() -> Self {
        Self {
            duration: Default::default(),
            function: EaseFunction::Linear,
            lifetime: Default::default(),
            pattern: Default::default(),
            cycles: Default::default(),
            direction: Default::default(),
            state: Default::default(),
            progress: Default::default(),
        }
    }
}

impl Tween {
    /// Reverses the tween according to the current progress by flipping the direction and mirroring the progress.
    ///
    /// This is useful for tween effects like hovering the cursor over a button:
    /// if the cursor leaves the button before the tween completes, it reverses from the current state.
    pub fn reverse(&mut self) {
        self.direction = self.direction.to_opposite();
        self.progress = 1.0 - self.progress;
    }
}

/// Added to a tween entity alongside [`Tween`] to provide the mutators for the tween.
///
/// This is parameterised by [`TweenType`], ensuring all [`TweenMutator`]s added are compatible with the [`TweenSubject`].
#[derive(Component)]
pub struct TweenMutators<T: TweenType> {
    values: Vec<
        Arc<
            dyn TweenMutator<Subject = <T::Param as TweenSystemParam>::Subject>
                + Send
                + Sync
                + 'static,
        >,
    >,
}

impl<T: TweenType> Default for TweenMutators<T> {
    fn default() -> Self {
        Self {
            values: Default::default(),
        }
    }
}

impl<T: TweenType> Clone for TweenMutators<T> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
        }
    }
}

impl<T, M> From<M> for TweenMutators<T>
where
    T: TweenType,
    M: TweenMutator<Subject = <T::Param as TweenSystemParam>::Subject> + Send + Sync + 'static,
{
    /// Creates a [`TweenMutators`] with a single mutator.
    fn from(value: M) -> Self {
        Self::default().with(value)
    }
}

impl<T: TweenType> TweenMutators<T> {
    /// Adds a mutator to the collection.
    pub fn with<M>(mut self, mutator: M) -> Self
    where
        M: TweenMutator<Subject = <T::Param as TweenSystemParam>::Subject> + Send + Sync + 'static,
    {
        self.values.push(Arc::new(mutator));
        self
    }
}

/// The entity containing the tween.
#[derive(Component, Clone)]
#[relationship(relationship_target = Tweens)]
pub struct TweenOf(pub Entity);

impl Default for TweenOf {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

/// Exists on an entity that is being tweened by a [`TweenOf`] entity.
///
/// It is "linked_spawn" so tweens related to the entity are despawned when the entity is.
#[derive(Component)]
#[relationship_target(relationship = TweenOf, linked_spawn)]
pub struct Tweens(Vec<Entity>);

/// Progresses the timing portion of the tweens.
fn progress_tweens(time: Res<Time>, mut tween_query: Query<&mut Tween>) {
    for mut tween in tween_query.iter_mut() {
        match tween.state {
            TweenState::Init => tween.state = TweenState::Running,
            TweenState::Running => {
                // apply progress since last frame
                tween.progress += time.delta_secs() / tween.duration.as_secs_f32();

                // loop until all the excess progress is consumed
                // this defends against possible extremely large delta times, or extremely small durations
                if tween.progress >= 1.0 {
                    loop {
                        if tween.cycles.is_exhausted() {
                            tween.progress = 1.0;
                            break;
                        }

                        tween.progress -= 1.0;
                        tween.cycles.decrement();

                        if tween.progress < 1.0 {
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Mutates the subjects of the tweens according to their type.
fn mutate_subjects<T: TweenType + 'static>(
    tween_query: Query<(&TweenOf, &mut Tween, &TweenMutators<T>)>,
    mut param: StaticSystemParam<T::Param>,
) {
    for (tween_of, tween, mutator) in tween_query.iter() {
        let TweenState::Running = tween.state else {
            continue;
        };

        let Some(mut subject) = T::Param::get_subject_mut(&mut param, tween_of.0) else {
            continue;
        };

        // get the fundamental progress dictated by overall direction
        let progress = match tween.direction {
            TweenDirection::Forward => tween.progress,
            TweenDirection::Backward => 1.0 - tween.progress,
        };

        // sample the function
        let sample = match tween.pattern {
            // single tweens sample the function with the unchanged progress
            TweenPattern::Single => tween.function.sample_clamped(progress),
            // two-step patterns change the sampling behaviour
            TweenPattern::PingPong | TweenPattern::ForwardReverse => {
                // two-step tweens fit the two legs of the tween into one duration
                // therefore, progress of 1.0 is considered to account for two "lengths" of the computed tween
                if progress < 0.5 {
                    tween.function.sample_clamped(progress * 2.0) // the first half is just like a single tween
                } else {
                    let progress = (progress - 0.5) * 2.0; // second half of progress normalised
                    if tween.pattern == TweenPattern::PingPong {
                        // the return journey is the inverse of the sample for ping-pong
                        1.0 - tween.function.sample_clamped(progress)
                    } else {
                        // otherwise, the return journey is the inverse of the progress
                        tween.function.sample_clamped(1.0 - progress)
                    }
                }
            }
        };

        for mutator in mutator.values.iter() {
            mutator.mutate(&mut subject, sample);
        }
    }
}

/// Handles tweens finishing, including triggering events and despawning of transient tweens.
fn finish_tweens(state_query: Query<(Entity, &Tween, &TweenOf)>, mut commands: Commands) {
    for (entity, tween, tween_of) in state_query.iter() {
        if tween.state == TweenState::Init {
            continue;
        }

        if tween.cycles.is_exhausted() {
            commands.trigger(TweenFinished {
                tween_entity: entity,
                tweened_entity: tween_of.0,
            });

            if tween.lifetime == TweenLifetime::Transient {
                // its possible that the handler for the finish event already despawns this, so try_despawn here
                commands.entity(entity).try_despawn();
            }
        }
    }
}
