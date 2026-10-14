use crate::core::*;
use bevy::{
    asset::AssetMut,
    ecs::system::{SystemParam, SystemParamItem, lifetimeless::Read},
    prelude::*,
};
use std::{marker::PhantomData, ops::Deref};

pub struct AssetTweenStrategy<A: Asset + 'static, R: Component + Deref<Target = Handle<A>>> {
    _phantom: PhantomData<(A, R)>,
}

impl<A: Asset + 'static, R: Component + Deref<Target = Handle<A>>> TweenStrategy
    for AssetTweenStrategy<A, R>
{
    type Param = AssetTweenParam<'static, 'static, A, R>;
}

#[derive(SystemParam)]
pub struct AssetTweenParam<'w, 's, A, R>
where
    A: Asset + 'static,
    R: Component + Deref<Target = Handle<A>>,
{
    assets: ResMut<'w, Assets<A>>,
    reference_query: Query<'w, 's, Read<R>>,
}

impl<'w, 's, A: Asset + 'static, R: Component + Deref<Target = Handle<A>>> TweenSystemParam
    for AssetTweenParam<'w, 's, A, R>
{
    type Subject<'sw, 'ss> = AssetMut<'sw, A>;

    fn get_subject_mut<'a, 'fw, 'fs>(
        item: &'a mut SystemParamItem<'fw, 'fs, Self>,
        entity: Entity,
    ) -> Option<<Self as TweenSystemParam>::Subject<'a, 'fs>> {
        item.reference_query
            .get(entity)
            .ok()
            .and_then(|r| item.assets.get_mut(r.id()))
    }
}
