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
use bevy_time::prelude::*;

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

pub trait GetTweenTypePlugin<T: TweenType> {
    fn plugin() -> TweenTypePlugin<T> {
        TweenTypePlugin::<T>::default()
    }
}

impl<T: TweenType> GetTweenTypePlugin<T> for T {}

/// Contains the systems in this plugin.
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum TweenSystems {
    Progress,
    Mutate,
    Finish,
}

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

pub trait TweenType {
    type Param: TweenSystemParam;

    fn configure_app(#[allow(unused)] app: &mut App) {}
}

pub trait TweenSubject {
    type Item<'w, 's>;
}

pub trait TweenMutatorImpl {
    type Subject: TweenSubject;

    fn mutate(&self, subject: <Self::Subject as TweenSubject>::Item<'_, '_>, factor: f32);
}

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
    // The entity responsible for the tween.
    #[event_target]
    pub tween_entity: Entity,
    pub tweened_entity: Entity,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TweenLifetime {
    #[default]
    Transient,
    Retained,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TweenPattern {
    #[default]
    Single,
    PingPong,
    ForwardReverse,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TweenCycles {
    Finite(u32),
    Infinite,
}

impl TweenCycles {
    pub fn zero() -> Self {
        Self::Finite(0)
    }

    pub fn once() -> Self {
        Self::Finite(1)
    }

    pub fn twice() -> Self {
        Self::Finite(2)
    }

    pub fn is_exhausted(&self) -> bool {
        match self {
            TweenCycles::Finite(count) if *count == 0 => true,
            _ => false,
        }
    }
}

impl Default for TweenCycles {
    fn default() -> Self {
        Self::once()
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TweenDirection {
    #[default]
    Forward,
    Backward,
}

impl TweenDirection {
    fn to_opposite(&self) -> Self {
        match self {
            TweenDirection::Forward => Self::Backward,
            TweenDirection::Backward => Self::Forward,
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum TweenState {
    #[default]
    Init,
    Running,
}

/// Contains the tween data independent of the tween type.
#[derive(Component, Clone, Debug)]
pub struct Tween {
    pub duration: Duration,
    pub function: EaseFunction,
    pub lifetime: TweenLifetime,
    pub pattern: TweenPattern,
    pub cycles: TweenCycles,
    pub direction: TweenDirection,
    pub state: TweenState,
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
    pub fn reverse(&mut self) {
        self.direction = self.direction.to_opposite();
        self.progress = 1.0 - self.progress;
    }
}

#[derive(Component, Clone)]
pub struct TweenMutator<S: TweenSubject> {
    value: Arc<dyn TweenMutatorImpl<Subject = S> + Send + Sync + 'static>,
}

impl<M: TweenMutatorImpl + Send + Sync + 'static> From<M> for TweenMutator<M::Subject> {
    fn from(value: M) -> Self {
        TweenMutator {
            value: Arc::new(value),
        }
    }
}

impl<S: TweenSubject + 'static> Default for TweenMutator<S> {
    fn default() -> Self {
        DefaultTweenMutatorImpl::<S> {
            _marker: PhantomData,
        }
        .into()
    }
}

struct DefaultTweenMutatorImpl<S> {
    _marker: PhantomData<fn() -> S>,
}

impl<S: TweenSubject> TweenMutatorImpl for DefaultTweenMutatorImpl<S> {
    type Subject = S;

    fn mutate(&self, _: <Self::Subject as TweenSubject>::Item<'_, '_>, _: f32) {}
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
#[derive(Component)]
#[relationship_target(relationship = TweenOf, linked_spawn)]
pub struct Tweens(Vec<Entity>);

/// Progresses the timing portion of the tweens.
fn progress_tweens(time: Res<Time>, mut tween_query: Query<&mut Tween>) {
    for mut tween in tween_query.iter_mut() {
        match tween.state {
            TweenState::Init => tween.state = TweenState::Running,
            TweenState::Running => {
                // normalize the progress between 0 and 1
                tween.progress += time.delta_secs() / tween.duration.as_secs_f32();
                if tween.progress >= 1.0 {
                    if let TweenCycles::Finite(count) = &mut tween.cycles {
                        if *count != 0 {
                            *count -= 1;
                        }
                    }

                    match tween.cycles {
                        TweenCycles::Finite(count) if count == 0 => {
                            tween.progress = 1.0;
                        }
                        TweenCycles::Infinite | TweenCycles::Finite(_) => loop {
                            tween.progress -= 1.0;

                            match tween.pattern {
                                TweenPattern::PingPong | TweenPattern::ForwardReverse => {
                                    tween.direction = tween.direction.to_opposite()
                                }
                                _ => (),
                            }

                            if tween.progress < 1.0 {
                                break;
                            }
                        },
                    }
                }
            }
        }
    }
}

/// Mutates the subjects of the tweens according to their type.
fn mutate_subjects<T: TweenType>(
    tween_query: Query<(
        &TweenOf,
        &mut Tween,
        &TweenMutator<<T::Param as TweenSystemParam>::Subject>,
    )>,
    mut param: StaticSystemParam<T::Param>,
) {
    for (tween_of, tween, mutator) in tween_query.iter() {
        let TweenState::Running = tween.state else {
            continue;
        };

        let Some(subject) = T::Param::get_subject_mut(&mut param, tween_of.0) else {
            continue;
        };

        // ping-pong uses a different strategy for backward, so we retain the progress
        let progress = match tween.direction {
            TweenDirection::Forward => tween.progress,
            TweenDirection::Backward => match tween.pattern {
                TweenPattern::PingPong => tween.progress,
                _ => 1.0 - tween.progress,
            },
        };

        // sample correctly: if we are on the backward step and the pattern is ping-pong, we need to reverse the sample
        let Some(sample) = (match tween.pattern {
            TweenPattern::PingPong if tween.direction == TweenDirection::Backward => {
                tween.function.sample(progress).map(|s| 1.0 - s)
            }
            _ => tween.function.sample(progress),
        }) else {
            continue;
        };

        mutator.value.mutate(subject, sample);
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
