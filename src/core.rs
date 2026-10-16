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
pub enum TweenLifecycle {
    #[default]
    OnceTransient,
    OnceRetained,
    Repeat,
    PingPong,
}

impl TweenLifecycle {
    pub fn is_transient(&self) -> bool {
        match self {
            Self::OnceTransient => true,
            _ => false,
        }
    }

    pub fn is_retained(&self) -> bool {
        !self.is_transient()
    }

    pub fn repeats(&self) -> bool {
        match self {
            Self::Repeat | Self::PingPong => true,
            _ => false,
        }
    }

    pub fn is_once(&self) -> bool {
        match self {
            Self::OnceTransient | Self::OnceRetained => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TweenDirection {
    #[default]
    Forwards,
    Backwards,
}

impl TweenDirection {
    fn to_opposite(&self) -> Self {
        match self {
            TweenDirection::Forwards => Self::Backwards,
            TweenDirection::Backwards => Self::Forwards,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub enum TweenState {
    #[default]
    NotStarted,
    Running,
    Finished,
}

/// Contains the tween data independent of the tween type.
#[derive(Component, Clone, Debug)]
pub struct Tween {
    pub duration: Duration,
    pub function: EaseFunction,
    pub lifecycle: TweenLifecycle,
    pub direction: TweenDirection,
    pub state: TweenState,
    pub progress: f32,
}

impl Default for Tween {
    fn default() -> Self {
        Self {
            duration: Default::default(),
            function: EaseFunction::Linear,
            lifecycle: Default::default(),
            direction: Default::default(),
            state: Default::default(),
            progress: Default::default(),
        }
    }
}

impl Tween {
    /// Reverses the tween according to the current progress, bouncing from the current position.
    pub fn bounce(&mut self) {
        self.direction = self.direction.to_opposite();
        self.progress = 1.0 - self.progress;
    }
}

impl Tween {
    pub fn new(duration: Duration, function: EaseFunction) -> Self {
        Self {
            duration,
            function,
            ..Default::default()
        }
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
            TweenState::NotStarted => tween.state = TweenState::Running,
            TweenState::Running => {
                // normalize the progress between 0 and 1
                tween.progress += time.delta_secs() / tween.duration.as_secs_f32();
                if tween.progress >= 1.0 {
                    match tween.lifecycle {
                        TweenLifecycle::OnceTransient | TweenLifecycle::OnceRetained => {
                            tween.progress = 1.0
                        }
                        TweenLifecycle::Repeat => tween.progress %= 1.0,
                        TweenLifecycle::PingPong => loop {
                            tween.progress -= 1.0;
                            tween.direction = tween.direction.to_opposite();
                            if tween.progress < 1.0 {
                                break;
                            }
                        },
                    }
                }
            }
            _ => (),
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

        let mut progress = tween.progress;

        // flip the progress if we are going backwards
        if tween.direction == TweenDirection::Backwards {
            progress = 1.0 - progress;
        }

        mutator
            .value
            .mutate(subject, tween.function.sample_unchecked(progress));
    }
}

/// Handles tweens finishing, including triggering events and despawning of transient tweens.
fn finish_tweens(mut state_query: Query<(Entity, &mut Tween, &TweenOf)>, mut commands: Commands) {
    for (entity, mut tween, tween_of) in state_query.iter_mut() {
        if let TweenState::Running = tween.state
            && tween.lifecycle.is_once()
            && tween.progress >= 1.0
        {
            tween.state = TweenState::Finished;
            commands.trigger(TweenFinished {
                tween_entity: entity,
                tweened_entity: tween_of.0,
            });
        }

        if let TweenState::Finished = tween.state
            && tween.lifecycle.is_transient()
        {
            // its possible that the handler for the finish event already despawns this, so try_despawn here
            commands.entity(entity).try_despawn();
        }
    }
}
