use std::{
    any::TypeId,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    time::Duration,
};

use bevy::{
    ecs::system::{StaticSystemParam, SystemParam, SystemParamItem},
    prelude::*,
};

pub struct TweenPlugin;

impl Plugin for TweenPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PreUpdate,
            (TweenSystems::TriggerEvents, TweenSystems::Cleanup).chain(),
        );

        app.configure_sets(
            PostUpdate,
            (TweenSystems::Progress, TweenSystems::Mutate).chain(),
        );

        app.add_systems(
            PreUpdate,
            trigger_events.in_set(TweenSystems::TriggerEvents),
        );
        app.add_systems(PreUpdate, cleanup_tweens.in_set(TweenSystems::Cleanup));

        app.add_systems(PostUpdate, progress_tweens.in_set(TweenSystems::Progress));
    }
}

pub struct TweenTypePlugin<T: TweenType> {
    _phantom: PhantomData<T>,
}

impl<T: TweenType> Default for TweenTypePlugin<T> {
    fn default() -> Self {
        Self {
            _phantom: default(),
        }
    }
}

impl<T: TweenType + Component> TweenTypePlugin<T> {
    fn mutate_subjects(
        tween_query: Query<(&TweenOf, &mut TweenState, &T)>,
        mut param: StaticSystemParam<<T::Strategy as TweenStrategy>::Param>,
    ) {
        for (tweener, state, tween) in tween_query.iter() {
            let Some(mut progress) = state.progress else {
                continue;
            };

            let Some(subject) =
                <T::Strategy as TweenStrategy>::Param::get_subject_mut(&mut param, tweener.0)
            else {
                continue;
            };

            // flip the progress if we are going backwards
            if state.direction == TweenDirection::Backwards {
                progress = 1.0 - progress;
            }

            tween.mutate_subject(subject, state.function.sample_unchecked(progress));
        }
    }
}

impl<T: TweenType + Component + Send + Sync + 'static> Plugin for TweenTypePlugin<T> {
    fn build(&self, app: &mut App) {
        T::mutate_app(app);

        app.configure_sets(
            PostUpdate,
            TweenTypeSystems::<T>::default().in_set(TweenSystems::Mutate),
        );

        app.add_systems(
            PostUpdate,
            Self::mutate_subjects.in_set(TweenTypeSystems::<T>::default()),
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
    TriggerEvents,
    Cleanup,
}

pub struct TweenTypeSystems<T: TweenType> {
    _marker: PhantomData<T>,
}

impl<T: TweenType> Default for TweenTypeSystems<T> {
    fn default() -> Self {
        Self { _marker: default() }
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
    type Strategy: TweenStrategy;

    fn mutate_subject(
        &self,
        item: <<Self::Strategy as TweenStrategy>::Param as TweenSystemParam>::Subject<'_, '_>,
        factor: f32,
    );

    fn mutate_app(#[allow(unused)] app: &mut App) {}
}

pub trait TweenStrategy {
    type Param: TweenSystemParam;
}

pub trait TweenSystemParam: SystemParam {
    /// The type of the subject that is retrieved from the system param.
    type Subject<'sw, 'ss>;

    /// Gets the subject from the system parameter.
    fn get_subject_mut<'a, 'w, 's>(
        item: &'a mut SystemParamItem<'w, 's, Self>,
        entity: Entity,
    ) -> Option<<Self as TweenSystemParam>::Subject<'a, 's>>;
}

/// Triggered when a tween finishes.
#[derive(EntityEvent)]
pub struct TweenFinished {
    // The entity responsible for the tween.
    #[event_target]
    pub tween_entity: Entity,
    pub tweened_entity: Entity,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TweenLifecycle {
    #[default]
    Once,
    Repeat,
    BounceRepeat,
}

impl TweenLifecycle {
    pub fn repeats(&self) -> bool {
        match self {
            Self::Repeat | Self::BounceRepeat => true,
            _ => false,
        }
    }

    pub fn is_one_shot(&self) -> bool {
        !self.repeats()
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum TweenDirection {
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

/// Contains the tween state independent of the tween type.
#[derive(Component, Clone)]
pub struct TweenState {
    pub duration: Duration,
    pub function: EaseFunction,
    pub lifecycle: TweenLifecycle,
    direction: TweenDirection,
    progress: Option<f32>,
}

impl Default for TweenState {
    fn default() -> Self {
        Self {
            duration: default(),
            function: EaseFunction::Linear,
            lifecycle: default(),
            direction: default(),
            progress: default(),
        }
    }
}

impl TweenState {
    pub fn new(duration: Duration, function: EaseFunction) -> Self {
        Self {
            duration,
            function,
            ..default()
        }
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
#[derive(Component)]
#[relationship_target(relationship = TweenOf, linked_spawn)]
pub struct Tweens(Vec<Entity>);

/// Progresses the timing portion of the tweens.
fn progress_tweens(time: Res<Time>, mut state_query: Query<&mut TweenState>) {
    for mut state in state_query.iter_mut() {
        match state.progress.is_none() {
            true => {
                // the tween is not subject to time passed this frame because it wasn't present in the last frame
                // this effectively initialises it to the start value of the tween type
                state.progress = Some(0.0);
            }
            false => {
                let mut progress = state.progress.unwrap(); // checked with match above

                // normalize the progress between 0 and 1
                progress += time.delta_secs() / state.duration.as_secs_f32();
                if progress >= 1.0 {
                    match state.lifecycle {
                        TweenLifecycle::Once => progress = 1.0,
                        TweenLifecycle::Repeat => progress %= 1.0,
                        TweenLifecycle::BounceRepeat => loop {
                            progress -= 1.0;
                            state.direction = state.direction.to_opposite();
                            if progress < 1.0 {
                                break;
                            }
                        },
                    }
                }

                state.progress = Some(progress);
            }
        }
    }
}

/// Triggers the [`TweenFinished`] event when one-shot tweens have finished.
fn trigger_events(state_query: Query<(Entity, &TweenState, &TweenOf)>, mut commands: Commands) {
    for (entity, state, tween_of) in state_query.iter() {
        let Some(progress) = state.progress else {
            continue;
        };

        if progress >= 1.0 && state.lifecycle.is_one_shot() {
            commands.trigger(TweenFinished {
                tween_entity: entity,
                tweened_entity: tween_of.0,
            });
        }
    }
}

/// Cleans up after one-shot tweens have completed by despawning complete entities.
fn cleanup_tweens(state_query: Query<(Entity, &TweenState)>, mut commands: Commands) {
    for (entity, state) in state_query.iter() {
        let Some(progress) = state.progress else {
            continue;
        };

        // despawn if the tween is complete
        if progress >= 1.0 && state.lifecycle.is_one_shot() {
            commands.entity(entity).despawn();
        }
    }
}
