use bevy::prelude::*;

use crate::{TimeSpan, TimeSpanProgress};

#[derive(Component)]
pub struct SkipTimeSpanGroup;

#[derive(Component)]
pub struct TimeSpanGroup;

pub fn time_span_group_system(
    q_group: Query<
        (&TimeSpanProgress, &TimeSpan, &Children),
        (With<TimeSpanGroup>, Without<SkipTimeSpanGroup>),
    >,
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
