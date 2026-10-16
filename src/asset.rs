use crate::core::*;
use bevy_asset::{AssetMut, prelude::*};
use bevy_ecs::{
    prelude::*,
    system::{SystemParam, SystemParamItem, lifetimeless::Read},
};
use std::{marker::PhantomData, ops::Deref};

#[derive(SystemParam)]
pub struct AssetTweenParam<'w, 's, A, R>
where
    A: Asset,
    R: Component + Deref<Target = Handle<A>>,
{
    assets: ResMut<'w, Assets<A>>,
    reference_query: Query<'w, 's, Read<R>>,
}

impl<A: Asset + 'static, R: Component + Deref<Target = Handle<A>>> TweenSystemParam
    for AssetTweenParam<'_, '_, A, R>
{
    type Subject = AssetTweenSubject<A>;

    fn get_subject_mut<'a, 'w, 's>(
        item: &'a mut SystemParamItem<'w, 's, Self>,
        entity: Entity,
    ) -> Option<<Self::Subject as TweenSubject>::Item<'a, 's>> {
        item.reference_query
            .get(entity)
            .ok()
            .and_then(|r| item.assets.get_mut(r.id()))
    }
}

pub struct AssetTweenSubject<A: Asset> {
    _phantom: PhantomData<fn() -> A>,
}

impl<A: Asset> TweenSubject for AssetTweenSubject<A> {
    type Item<'w, 's> = AssetMut<'w, A>;
}

impl<A: Asset> Clone for AssetTweenSubject<A> {
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}
