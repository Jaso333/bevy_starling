use std::{marker::PhantomData, ops::Deref, time::Duration};

use bevy::{
    asset::AssetMut,
    ecs::{
        query::QueryData,
        system::{StaticSystemParam, SystemParam, SystemParamItem},
    },
    prelude::*,
};

pub mod prelude {
    pub use crate::{
        MeshMaterial3dStandardMaterialBaseColorTween, TransformTranslationTween, TweenOf,
        TweenPlugin, TweenState, TweenSystems, TweenType, TweenTypePlugin, Tweens,
    };
}

/// Contains the systems in this plugin.
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum TweenSystems {
    Progress,
    Mutate,
    Cleanup,
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
    progress: f32,
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
pub struct TweenPlugin;

impl Plugin for TweenPlugin {
    fn build(&self, app: &mut App) {
        // Update is selected as the schedule because tweens generally form part of spatial game logic:
        // you might detect something that causes a tween, but equally you might need to react to a tween moving something.
        app.configure_sets(
            Update,
            (
                TweenSystems::Progress,
                TweenSystems::Mutate,
                TweenSystems::Cleanup,
            )
                .chain(),
        )
        .configure_sets(
            Update,
            TweenSystems::Cleanup.before(TransformSystems::Propagate),
        )
        .add_systems(Update, progress_tweens.in_set(TweenSystems::Progress))
        .add_systems(Update, cleanup_tweens.in_set(TweenSystems::Cleanup));
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

impl<T: TweenType<M> + Component, M: Send + Sync + 'static> Plugin for TweenTypePlugin<T, M> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            mutate_tween_subjects::<T, M>.in_set(TweenSystems::Mutate),
        );
    }
}

/// Progresses the timing portion of the tweens.
fn progress_tweens(time: Res<Time>, mut state_query: Query<&mut TweenState>) {
    for mut state in state_query.iter_mut() {
        // normalize the progress between 0 and 1
        state.progress += time.delta_secs() / state.duration.as_secs_f32();
        if state.progress >= 1.0 {
            state.progress = 1.0;
        }
    }
}

/// Performs the tween mutations.
fn mutate_tween_subjects<T: TweenType<M> + Component, M>(
    tween_query: Query<(&TweenOf, &mut TweenState, &T)>,
    mut param: StaticSystemParam<T::Param>,
) {
    for (tweener, tween_state, tween) in tween_query.iter() {
        let Some(subject) = T::Param::get_subject_mut(&mut param, tweener.0) else {
            continue;
        };

        tween.mutate_subject(
            subject,
            tween_state.function.sample_unchecked(tween_state.progress),
        );
    }
}

/// Cleans up after tweens have completed by despawning complete entities.
fn cleanup_tweens(state_query: Query<(Entity, &TweenState)>, mut commands: Commands) {
    for (entity, state) in state_query.iter() {
        // despawn if the tween is complete
        if state.progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// The actual tween component that is specific to a type of tween.
pub trait TweenType<M>: Sized {
    /// The parameter used for the mutation system.
    type Param: TweenTypeSystemParam;

    /// Mutates the subject of the tween by the given factor.
    fn mutate_subject(
        &self,
        item: <Self::Param as TweenTypeSystemParam>::Subject<'_, '_>,
        factor: f32,
    );

    /// Provides the plugin for this tween type.
    fn plugin() -> TweenTypePlugin<Self, M> {
        TweenTypePlugin::new()
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
pub trait QueryDataTween: Component {
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
pub trait EntityAssetTween: Component {
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

    fn plugin() -> TweenTypePlugin<Self, EntityAssetTweenStrategy> {
        TweenTypePlugin::new()
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
    type Data = &'static mut Transform;

    fn mutate_data(&self, mut item: <Self::Data as QueryData>::Item<'_, '_>, factor: f32) {
        item.translation = self.start.lerp(self.end, factor);
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
