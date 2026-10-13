use std::{
    any::TypeId,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    time::Duration,
};

use bevy::{
    asset::AssetMut,
    ecs::{
        query::QueryData,
        system::{StaticSystemParam, SystemParam, SystemParamItem, lifetimeless::Write},
    },
    prelude::*,
};

/// Contains the systems in this plugin.
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum TweenSystems {
    Progress,
    Mutate,
    TriggerEvents,
    Cleanup,
}

/// System set for tween types.
pub struct TweenTypeSystems<T: TweenType<M>, M> {
    _marker: PhantomData<(T, M)>,
}

impl<T: TweenType<M>, M: Send + Sync + 'static> SystemSet for TweenTypeSystems<T, M> {
    fn dyn_clone(&self) -> Box<dyn SystemSet> {
        todo!()
    }
}

impl<T: TweenType<M>, M> Debug for TweenTypeSystems<T, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TweenTypeSystems")
            .field("_marker", &self._marker)
            .finish()
    }
}

impl<T: TweenType<M>, M: 'static> Hash for TweenTypeSystems<T, M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        TypeId::of::<T>().hash(state);
        TypeId::of::<M>().hash(state);
    }
}

impl<T: TweenType<M>, M> PartialEq for TweenTypeSystems<T, M> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T: TweenType<M>, M> Eq for TweenTypeSystems<T, M> {}

impl<T: TweenType<M>, M> Default for TweenTypeSystems<T, M> {
    fn default() -> Self {
        Self { _marker: default() }
    }
}

impl<T: TweenType<M>, M> Clone for TweenTypeSystems<T, M> {
    fn clone(&self) -> Self {
        Self {
            _marker: self._marker.clone(),
        }
    }
}

/// Triggered when a tween finishes.
#[derive(EntityEvent)]
pub struct TweenFinished {
    // The entity responsible for the tween.
    #[event_target]
    pub tween_entity: Entity,
    pub tweened_entity: Option<Entity>,
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

/// Contains the tween state independent of the tween type.
#[derive(Component, Clone)]
pub struct TweenState {
    pub duration: Duration,
    pub function: EaseFunction,
    progress: Option<f32>,
}

impl Default for TweenState {
    fn default() -> Self {
        Self {
            duration: default(),
            function: EaseFunction::Linear,
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

/// Adds the base tween functionality.
///
/// This feature has been designed with the following guarantees (assuming tweens are spawned during the [`Update`] schedule):
/// - On the first frame that a tween exists, the subject of the tween will be equal to the start value.
/// - On the last frame that a tween exists, the subject of the tween will be equal to the end value.
///
/// For this guarantee to be visualized, the propagation of any such subject must happen after the [`TweenSystems::Mutate`] set.
/// For example, transform tweens will be appropriately reflected in the next frame render because the systems in this plugin are ordered before [`TransformSystems::Propagate`].
///
/// This guarantee means that a theoretical zero-duration tween will span across two frames: the first frame showing the subject start value, and the second showing the subject end value.
pub struct TweenPlugin;

impl Plugin for TweenPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PreUpdate,
            (TweenSystems::TriggerEvents, TweenSystems::Cleanup).chain(),
        );

        app.configure_sets(
            PostUpdate,
            (TweenSystems::Progress, TweenSystems::Mutate)
                .chain()
                .before(TransformSystems::Propagate),
        );

        app.add_systems(
            PreUpdate,
            trigger_events.in_set(TweenSystems::TriggerEvents),
        );
        app.add_systems(PreUpdate, cleanup_tweens.in_set(TweenSystems::Cleanup));

        app.add_systems(PostUpdate, progress_tweens.in_set(TweenSystems::Progress));
    }
}

/// Declares a plugin for an implemented [`TweenType`].
/// T is the [`TweenType`] and M is the strategy marker type that allows trait specialization.
pub struct TweenTypePlugin<T, M> {
    _marker: PhantomData<(T, M)>,
}

impl<T, M> TweenTypePlugin<T, M> {
    pub fn new() -> Self {
        Self { _marker: default() }
    }
}

impl<T: TweenType<M> + Component, M> TweenTypePlugin<T, M> {
    /// Performs the tween mutations. This is automatically added to the app by the plugin.
    pub fn mutate_tween_subjects(
        tween_query: Query<(&TweenOf, &mut TweenState, &T)>,
        mut param: StaticSystemParam<T::Param>,
    ) {
        for (tweener, state, tween) in tween_query.iter() {
            let Some(progress) = state.progress else {
                continue;
            };

            let Some(subject) = T::Param::get_subject_mut(&mut param, tweener.0) else {
                continue;
            };

            tween.mutate_subject(subject, state.function.sample_unchecked(progress));
        }
    }
}

impl<T: TweenType<M> + Component, M: Send + Sync + 'static> Plugin for TweenTypePlugin<T, M> {
    fn build(&self, app: &mut App) {
        let set = T::set();

        T::build(app);
        app.configure_sets(PostUpdate, set.clone().in_set(TweenSystems::Mutate));
        app.add_systems(PostUpdate, Self::mutate_tween_subjects.in_set(set.clone()));
    }
}

/// Allows the tween type to make configuration changes to the app as part of plugin building.
pub trait TweenTypePluginConfig {
    /// Provides an opportunity to modify the app on behalf of the plugin for this tween type.
    fn build(_app: &mut App) {}
}

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
                let duration = state.duration.as_secs_f32();
                let progress = state.progress.as_mut().unwrap(); // checked with match above

                // normalize the progress between 0 and 1
                *progress += time.delta_secs() / duration;
                if *progress >= 1.0 {
                    *progress = 1.0;
                }
            }
        }
    }
}

/// Triggers the [`TweenFinished`] event when the tweens have finished.
fn trigger_events(
    state_query: Query<(Entity, &TweenState, Option<&TweenOf>)>,
    mut commands: Commands,
) {
    for (entity, state, tween_of) in state_query.iter() {
        let Some(progress) = state.progress else {
            continue;
        };

        if progress >= 1.0 {
            commands.trigger(TweenFinished {
                tween_entity: entity,
                tweened_entity: tween_of.map(|to| to.0),
            });
        }
    }
}

/// Cleans up after tweens have completed by despawning complete entities.
fn cleanup_tweens(state_query: Query<(Entity, &TweenState)>, mut commands: Commands) {
    for (entity, state) in state_query.iter() {
        let Some(progress) = state.progress else {
            continue;
        };

        // despawn if the tween is complete
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// The actual tween component that is specific to a type of tween.
pub trait TweenType<M>: Sized + TweenTypePluginConfig + Component {
    /// The parameter used for the mutation system.
    type Param: TweenTypeSystemParam;

    /// Mutates the subject of the tween by the given factor.
    fn mutate_subject(
        &self,
        item: <Self::Param as TweenTypeSystemParam>::Subject<'_, '_>,
        factor: f32,
    );
}

/// Provides the plugin for a tween type.
pub trait GetTweenTypePlugin<T, M> {
    fn plugin() -> TweenTypePlugin<T, M>;
}

impl<T: TweenType<M>, M> GetTweenTypePlugin<T, M> for T {
    fn plugin() -> TweenTypePlugin<T, M> {
        TweenTypePlugin::new()
    }
}

/// Provides the system srt for the tween type.
pub trait GetTweenTypeSet<T: TweenType<M>, M> {
    fn set() -> TweenTypeSystems<T, M>;
}

impl<T: TweenType<M>, M> GetTweenTypeSet<T, M> for T {
    fn set() -> TweenTypeSystems<T, M> {
        TweenTypeSystems::default()
    }
}

/// The contract for a [`SystemParam`] that provides the subject for a tween.
pub trait TweenTypeSystemParam: SystemParam {
    /// The subject that is passed to the mutation function.
    type Subject<'a, 's>;

    /// Gets the subject from the system parameter.
    fn get_subject_mut<'a, 'w, 's>(
        item: &'a mut SystemParamItem<'w, 's, Self>,
        entity: Entity,
    ) -> Option<<Self as TweenTypeSystemParam>::Subject<'a, 's>>;
}

/// A tween type that mutates some query data.
pub trait QueryDataTween: Component + TweenTypePluginConfig {
    /// The data for the query to the entity that the tween points to.
    type Data: QueryData + 'static;

    /// Mutates the data provided by the query.
    fn mutate_data(&self, item: <Self::Data as QueryData>::Item<'_, '_>, factor: f32);
}

/// Marker for query data tween strategy.
pub struct QueryDataTweenStrategy;

impl<T: QueryDataTween> TweenType<QueryDataTweenStrategy> for T {
    type Param = QueryDataTweenSystemParam<'static, 'static, T::Data>;

    fn mutate_subject(
        &self,
        item: <Self::Param as TweenTypeSystemParam>::Subject<'_, '_>,
        factor: f32,
    ) {
        // call up to the more specific function
        self.mutate_data(item, factor);
    }
}

/// The system param containing items necessary for a query data tween.
#[derive(SystemParam)]
pub struct QueryDataTweenSystemParam<'w, 's, Q: QueryData + 'static> {
    subject_query: Query<'w, 's, Q>,
}

impl<'w0, 's0, Q: QueryData + 'static> TweenTypeSystemParam
    for QueryDataTweenSystemParam<'w0, 's0, Q>
{
    type Subject<'a, 's1> = Q::Item<'a, 's1>;

    fn get_subject_mut<'a, 'w1, 's1>(
        item: &'a mut SystemParamItem<'w1, 's1, Self>,
        entity: Entity,
    ) -> Option<<Self as TweenTypeSystemParam>::Subject<'a, 's1>> {
        // simply index into the query
        item.subject_query.get_mut(entity).ok()
    }
}

/// Tweens the asset referenced by an entity.
pub trait EntityAssetTween: Component + TweenTypePluginConfig {
    /// The asset being tweened.
    type Asset: Asset;

    /// The component that dereferences to a handle that identifies the asset.
    type HandleComponent: Component + Deref<Target = Handle<Self::Asset>>;

    // mutates the asset in question
    fn mutate_asset(&self, asset: AssetMut<'_, Self::Asset>, factor: f32);
}

/// Marker for entity asset tween strategy.
pub struct EntityAssetTweenStrategy;

impl<T: EntityAssetTween> TweenType<EntityAssetTweenStrategy> for T {
    type Param = EntityAssetTweenSystemParam<'static, 'static, T::HandleComponent, T::Asset>;

    fn mutate_subject(
        &self,
        item: <Self::Param as TweenTypeSystemParam>::Subject<'_, '_>,
        factor: f32,
    ) {
        // pass up
        self.mutate_asset(item, factor);
    }
}

/// The system param containing items necessary for an entity asset tween.
#[derive(SystemParam)]
pub struct EntityAssetTweenSystemParam<'w, 's, C: Component + Deref<Target = Handle<A>>, A: Asset> {
    subject_query: Query<'w, 's, &'static C>,
    asset_resource: ResMut<'w, Assets<A>>,
}

impl<C: Component + Deref<Target = Handle<A>>, A: Asset> TweenTypeSystemParam
    for EntityAssetTweenSystemParam<'_, '_, C, A>
{
    type Subject<'a, 's> = AssetMut<'a, A>;

    fn get_subject_mut<'a, 'w, 's>(
        item: &'a mut SystemParamItem<'w, 's, Self>,
        entity: Entity,
    ) -> Option<<Self as TweenTypeSystemParam>::Subject<'a, 's>> {
        // use the component to get the id, then get the asset using the id
        item.subject_query
            .get(entity)
            .ok()
            .map(|component| component.id())
            .and_then(|id| item.asset_resource.get_mut(id))
    }
}

/// Tweens the entire translation of a [`Transform`].
#[derive(Component, Default, Clone)]
pub struct TransformTranslationTween {
    pub start: Vec3,
    pub end: Vec3,
}

impl QueryDataTween for TransformTranslationTween {
    type Data = Write<Transform>;

    fn mutate_data(&self, mut item: <Self::Data as QueryData>::Item<'_, '_>, factor: f32) {
        item.translation = self.start.lerp(self.end, factor);
    }
}

impl TweenTypePluginConfig for TransformTranslationTween {
    fn build(app: &mut App) {
        app.configure_sets(PostUpdate, Self::set().before(TransformSystems::Propagate));
    }
}

/// Tweens the [`StandardMaterial::base_color`] of the [`StandardMaterial`] referenced by a [`MeshMaterial3d`].
#[derive(Component, Default, Clone)]
pub struct MeshMaterial3dStandardMaterialBaseColorTween {
    pub start: Color,
    pub end: Color,
}

impl EntityAssetTween for MeshMaterial3dStandardMaterialBaseColorTween {
    type Asset = StandardMaterial;
    type HandleComponent = MeshMaterial3d<StandardMaterial>;

    fn mutate_asset(&self, mut asset: AssetMut<'_, StandardMaterial>, factor: f32) {
        asset.base_color = self.start.mix(&self.end, factor);
    }
}

impl TweenTypePluginConfig for MeshMaterial3dStandardMaterialBaseColorTween {}
