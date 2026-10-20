use std::marker::PhantomData;

use crate::core::*;
use bevy_ecs::{
    prelude::*,
    query::QueryData,
    system::{SystemParam, SystemParamItem},
};

/// Generic parameter used for tweens that operate on [`QueryData`] as a subject.
///
/// Most tween types will reference this parameters.
#[derive(SystemParam)]
pub struct QueryTweenParam<'w, 's, D>
where
    D: QueryData + 'static,
{
    subject_query: Query<'w, 's, D>,
}

impl<D: QueryData + Send + Sync + 'static> TweenSystemParam for QueryTweenParam<'_, '_, D> {
    type Subject = QueryTweenSubject<D>;

    fn get_subject_mut<'a, 'w, 's>(
        item: &'a mut SystemParamItem<'w, 's, Self>,
        entity: Entity,
    ) -> Option<<Self::Subject as TweenSubject>::Item<'a, 's>> {
        item.subject_query.get_mut(entity).ok()
    }
}

pub struct QueryTweenSubject<D: QueryData> {
    _phantom: PhantomData<fn() -> D>,
}

impl<D: QueryData> TweenSubject for QueryTweenSubject<D> {
    type Item<'w, 's> = D::Item<'w, 's>;
}

impl<D: QueryData> Clone for QueryTweenSubject<D> {
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}
