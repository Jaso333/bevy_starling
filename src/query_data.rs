use crate::core::*;
use bevy::{
    ecs::{
        query::QueryData,
        system::{SystemParam, SystemParamItem},
    },
    prelude::*,
};
use std::marker::PhantomData;

pub struct QueryDataTweenStrategy<D: QueryData + 'static> {
    _phatom: PhantomData<D>,
}

impl<D: QueryData + 'static> TweenStrategy for QueryDataTweenStrategy<D> {
    type Param = QueryDataTweenParam<'static, 'static, D>;
}

#[derive(SystemParam)]
pub struct QueryDataTweenParam<'w, 's, D>
where
    D: QueryData + 'static,
{
    subject_query: Query<'w, 's, D>,
}

impl<D: QueryData + 'static> TweenSystemParam for QueryDataTweenParam<'_, '_, D> {
    type Subject<'sw, 'ss> = D::Item<'sw, 'ss>;

    fn get_subject_mut<'a, 'fw, 'fs>(
        item: &'a mut SystemParamItem<'fw, 'fs, Self>,
        entity: Entity,
    ) -> Option<<Self as TweenSystemParam>::Subject<'a, 'fs>> {
        item.subject_query.get_mut(entity).ok()
    }
}
