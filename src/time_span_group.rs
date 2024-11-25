use crate::{TimeSpan, TimeSpanProgress};
#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::{
    ReflectComponent, ReflectFromWorld, ReflectMapEntities, ReflectVisitEntities,
    ReflectVisitEntitiesMut,
};
use bevy_ecs::{entity::VisitEntitiesMut, prelude::*};
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use smallvec::SmallVec;

#[derive(Component, Debug, VisitEntitiesMut)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(
    feature = "bevy_reflect",
    reflect(
        Component,
        MapEntities,
        VisitEntities,
        VisitEntitiesMut,
        Debug,
        FromWorld
    )
)]
pub struct TimeSpanGroup(SmallVec<[Entity; 8]>);

impl<'a> IntoIterator for &'a TimeSpanGroup {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = std::slice::Iter<'a, Entity>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Default for TimeSpanGroup {
    fn default() -> Self {
        TimeSpanGroup(SmallVec::new())
    }
}

pub fn time_span_group_system(
    q_group: Query<(&TimeSpanProgress, &TimeSpan, &TimeSpanGroup)>,
    mut q_span: Query<(Entity, Option<&mut TimeSpanProgress>, &TimeSpan)>,
) {
    q_group.iter().for_each(|(progress, span, children)| {
        let sub_now = span.min().duration().as_secs_f32() - progress.now;
        let sub_previous = span.min().duration().as_secs_f32() - progress.now;
        let length = (span.max().duration() - span.min().duration()).as_secs_f32();
        for &child in children {
            let (entity, progress, span) = q_span.get(child).unwrap();

            let new_progress = TimeSpanProgress {
                now_percentage: todo!(),
                now: todo!(),
                previous_percentage: todo!(),
                previous: todo!(),
            };
        }
    });
}
