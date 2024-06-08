use bevy_ecs::prelude::*;
use bevy_hierarchy::prelude::*;
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bevy_time::prelude::*;
use std::{cmp::Ordering, time::Duration};

#[cfg(feature = "bevy_eventlistener")]
use bevy_eventlistener::prelude::*;

use crate::time_span::*;

/// Contains the current elasped time per tick.
/// Have more informations useful for handling edge cases and retain timing accuracy.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct TimeRunnerElasped {
    now: f32,
    now_period: f32,
    previous: f32,
    previous_period: f32,
}

impl TimeRunnerElasped {
    fn update(&mut self, now: f32, now_period: f32) {
        self.previous = self.now;
        self.previous_period = self.now_period;
        self.now = now;
        self.now_period = now_period;
    }

    /// The current elasped seconds. Always within timer's length.
    pub fn now(&self) -> f32 {
        self.now
    }
    /// Value between 0–1 as percentage of the timer period.
    /// Value may goes over or under 0–1 to indicate looping or repeating in
    /// arbitary times.
    pub fn now_period(&self) -> f32 {
        self.now_period
    }
    /// The previous elasped seconds. Always within timer's length.
    pub fn previous(&self) -> f32 {
        self.previous
    }
    /// Previous value between 0–1 as percentage of the timer period.
    /// Value may goes over or under 0–1 to indicate looping or repeating in
    /// arbitary times.
    pub fn previous_period(&self) -> f32 {
        self.previous_period
    }
}

/// Advanced timer
#[derive(Debug, Clone, PartialEq, Component)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
pub struct TimeRunner {
    paused: bool,
    /// The current elasped time with other useful information.
    elasped: TimeRunnerElasped,
    /// Maximum amount of duration.
    length: Duration,
    /// Ticking direction of the current timer.
    direction: TimeDirection,
    /// Time scale for ticking
    time_scale: f32,
    /// Repeat configuration.
    repeat: Option<(Repeat, RepeatStyle)>,
}

impl TimeRunner {
    /// Create new [`TimeRunner`] with this duration.
    pub fn new(length: Duration) -> TimeRunner {
        TimeRunner {
            length,
            ..Default::default()
        }
    }

    /// Set timer length
    pub fn set_length(&mut self, duration: Duration) -> &mut Self {
        self.length = duration;
        self
    }

    /// Get timer length
    pub fn length(&self) -> Duration {
        self.length
    }

    /// Pauses the timer.
    pub fn set_paused(&mut self, paused: bool) -> &mut Self {
        self.paused = paused;
        self
    }

    /// Get timer paused
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Set timer time scale
    pub fn set_time_scale(&mut self, time_scale: f32) -> &mut Self {
        self.time_scale = time_scale;
        self
    }

    /// Get timer time scale
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Set timer direction
    pub fn set_direction(&mut self, direction: TimeDirection) -> &mut Self {
        self.direction = direction;
        self
    }

    /// Get timer direction
    pub fn direction(&self) -> TimeDirection {
        self.direction
    }

    /// Set repeat
    pub fn set_repeat(&mut self, repeat: Option<(Repeat, RepeatStyle)>) -> &mut Self {
        self.repeat = repeat;
        self
    }

    /// Get timer repeat
    pub fn repeat(&self) -> Option<(Repeat, RepeatStyle)> {
        self.repeat
    }

    /// Get timer elasped time
    pub fn elasped(&self) -> TimeRunnerElasped {
        self.elasped
    }

    /// Returns true if the timer is completed.
    /// Completed meaning that there will be no more ticking and all
    /// configured repeat is exhausted.
    pub fn is_completed(&self) -> bool {
        let at_edge = match self.direction {
            TimeDirection::Forward => {
                self.elasped.now_period >= 1.0
                    && self.elasped.now_period == self.elasped.previous_period
            }
            TimeDirection::Backward => {
                self.elasped.now_period <= 0.0 && self.elasped.now == self.elasped.previous
            }
        };
        match self.repeat {
            Some((repeat, _)) => repeat.exhausted() && at_edge,
            None => at_edge,
        }
    }

    /// Update [`TimeRunnerElasped`] by `secs`.
    /// Accounted for `paused`, `time_scale` and if the timer is completed.
    ///
    /// # Panics
    ///
    /// Panics if `secs` is Nan.
    pub fn tick(&mut self, secs: f32) {
        if self.paused || self.is_completed() {
            return;
        }
        self.raw_tick(secs * self.time_scale);
    }

    /// Update [`TimeRunnerElasped`] by `secs`.
    /// Doesn't account for `paused`, `time_scale` and if the timer is completed.
    ///
    /// # Panics
    ///
    /// Panics if `secs` is Nan.
    pub fn raw_tick(&mut self, secs: f32) {
        use RepeatStyle::*;
        use TimeDirection::*;

        assert!(!secs.is_nan(), "Tick seconds can't be Nan");

        let length = self.length.as_secs_f32();
        let now = self.elasped.now;

        let new_elasped = match self.direction {
            Forward => now + secs,
            Backward => now - secs,
        };

        let p = period_percentage(new_elasped, length);

        let repeat_count = p.floor() as i32;
        let repeat_style = 'a: {
            if let Some(r) = self.repeat.as_mut() {
                if repeat_count != 0 {
                    let repeat_count = if self.direction == TimeDirection::Forward {
                        repeat_count
                    } else {
                        -repeat_count
                    };
                    let advances = r.0.advance_counter_by(repeat_count);
                    if advances != 0 {
                        break 'a r.1;
                    }
                }
            }
            if new_elasped > length {
                self.elasped.update(length, 1.);
            } else if new_elasped < 0. {
                self.elasped.update(0., 0.);
            } else {
                self.elasped.update(new_elasped, p);
            };
            return;
        };

        let new_elasped = match repeat_style {
            WrapAround => saw_wave(new_elasped, length),
            PingPong => triangle_wave(new_elasped, length),
        };
        self.elasped.update(new_elasped, p);

        if repeat_style == RepeatStyle::PingPong {
            let new_direction = match self.direction {
                Forward => triangle_wave_direction(repeat_count),
                Backward => backward_triangle_wave_direction(repeat_count),
            };
            self.direction = new_direction;
        }
    }

    /// Set currently elasped now to `secs`.
    pub fn set_tick(&mut self, secs: f32) {
        self.elasped.now = secs;
        self.elasped.now_period = period_percentage(secs, self.length.as_secs_f32());
    }

    /// Call this method when you've handled the range of time between `previous`
    /// and `now` inside [`TimerElasped`].
    /// Set all `previous` in [`TimerElasped`] to `now`.
    pub(crate) fn collaspe_elasped(&mut self) {
        self.elasped.previous = self.elasped.now;
        self.elasped.previous_period = self.elasped.now_period;
    }
}

impl Default for TimeRunner {
    fn default() -> Self {
        TimeRunner {
            paused: Default::default(),
            elasped: Default::default(),
            length: Default::default(),
            direction: Default::default(),
            time_scale: 1.,
            repeat: Default::default(),
        }
    }
}

/// Timer repeat configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum Repeat {
    /// Repeat infinitely
    Infinitely,
    /// Repeat infinitely and count the times this timer has repeated
    InfinitelyCounted {
        /// The times this timer has repeated
        times_repeated: i32,
    },
    /// Repeat for this amount of times
    Times {
        /// Times to repeat for
        #[allow(missing_docs)]
        times: i32,
        /// Times this timer has repeated.
        #[allow(missing_docs)]
        times_repeated: i32,
    },
}

impl Repeat {
    /// Repeat infinitely
    pub fn infinitely() -> Repeat {
        Repeat::Infinitely
    }

    /// Repeat infinitely and count the times this timer has repeated
    pub fn infinitely_counted() -> Repeat {
        Repeat::InfinitelyCounted { times_repeated: 0 }
    }

    /// Repeat for this amount of times
    pub fn times(times: i32) -> Repeat {
        Repeat::Times {
            times,
            times_repeated: 0,
        }
    }

    /// Returns if all repeat has been exhausted.
    /// Infinite repeat always returns false.
    pub fn exhausted(&self) -> bool {
        match self {
            Repeat::Infinitely => false,
            Repeat::InfinitelyCounted { .. } => false,
            Repeat::Times {
                times,
                times_repeated,
            } => times_repeated >= times,
        }
    }

    /// Returns actual advanced count.
    pub fn advance_counter_by(&mut self, by: i32) -> i32 {
        match self {
            Repeat::Infinitely => by,
            Repeat::InfinitelyCounted { times_repeated } => {
                *times_repeated += by;
                by
            }
            Repeat::Times {
                times,
                times_repeated,
            } => {
                let times_left = *times - *times_repeated;
                if times_left == 0 {
                    return 0;
                }
                let times_to_advance = if times_left > by { by } else { times_left };
                *times_repeated += times_to_advance;
                times_to_advance
            }
        }
    }
}

/// Time runner repeat behavior
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum RepeatStyle {
    /// Timer will wrap around.
    #[default]
    WrapAround,
    /// Timer will flip its direction.
    PingPong,
}

fn saw_wave(x: f32, period: f32) -> f32 {
    x.rem_euclid(period)
}

fn triangle_wave(x: f32, period: f32) -> f32 {
    ((x + period).rem_euclid(period * 2.) - period).abs()
}

fn triangle_wave_direction(repeats: i32) -> TimeDirection {
    if repeats.rem_euclid(2) == 0 {
        TimeDirection::Forward
    } else {
        TimeDirection::Backward
    }
}

fn backward_triangle_wave_direction(repeats: i32) -> TimeDirection {
    if repeats.rem_euclid(2) == 0 {
        TimeDirection::Backward
    } else {
        TimeDirection::Forward
    }
}

fn period_percentage(x: f32, period: f32) -> f32 {
    x / period
}

/// Skip a [`TimeRunner`].
#[derive(Debug, Clone, Copy, Component)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
pub struct SkipTimeRunner;

/// Fired when a time runner repeated or completed
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_eventlistener", derive(EntityEvent))]
#[cfg_attr(feature = "bevy_eventlistener", can_bubble)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Event)]
pub struct TimeRunnerEnded {
    /// [`TimeRunner`] that just ended
    #[cfg_attr(feature = "bevy_eventlistener", target)]
    pub time_runner: Entity,
    /// Currently timer direction. If is [`RepeatStyle::PingPong`], the current
    /// direction will be its already changed direction.
    pub current_direction: TimeDirection,
    /// The repeat this time runner had.
    pub with_repeat: Option<Repeat>,
}

impl TimeRunnerEnded {
    /// Returns true if the time runner is completed.
    /// Completed meaning that there will be no more ticking and all
    /// configured repeat is exhausted.
    pub fn is_completed(&self) -> bool {
        self.with_repeat
            .map(|repeat| repeat.exhausted())
            .unwrap_or(true)
    }
}

/// Tick time runner then send [`TimeRunnerEnded`] event if qualified for.
pub fn tick_time_runner_system(
    time: Res<Time<Real>>,
    mut q_time_runner: Query<(Entity, &mut TimeRunner)>,
    mut ended_writer: EventWriter<TimeRunnerEnded>,
) {
    let delta = time.delta_seconds();
    q_time_runner
        .iter_mut()
        .for_each(|(entity, mut time_runner)| {
            if time_runner.paused || time_runner.is_completed() {
                return;
            }
            let scale = time_runner.time_scale;
            time_runner.raw_tick(delta * scale);

            let n = time_runner.elasped().now_period;
            let send_event = match time_runner.repeat {
                Some((_, RepeatStyle::PingPong)) => {
                    (time_runner.direction == TimeDirection::Forward && n < 0.)
                        || (time_runner.direction == TimeDirection::Backward && n >= 1.)
                }
                _ => {
                    (time_runner.direction == TimeDirection::Backward && n < 0.)
                        || (time_runner.direction == TimeDirection::Forward && n >= 1.)
                }
            };
            if send_event {
                ended_writer.send(TimeRunnerEnded {
                    time_runner: entity,
                    current_direction: time_runner.direction,
                    with_repeat: time_runner.repeat.map(|r| r.0),
                });
            }
        });
}

/// System for updating any [`TimeSpan`] with the correct [`TimeSpanProgress`]
/// by their runner
pub fn time_runner_system(
    mut commands: Commands,
    mut q_runner: Query<(Entity, &mut TimeRunner, Option<&Children>), Without<SkipTimeRunner>>,
    mut q_span: Query<(Entity, Option<&mut TimeSpanProgress>, &TimeSpan)>,
    q_added_skip: Query<(Entity, &TimeRunner, Option<&Children>), Added<SkipTimeRunner>>,
    mut runner_just_completed: Local<Vec<Entity>>,
) {
    use DurationQuotient::*;
    use RepeatStyle::*;
    use TimeDirection::*;

    let mut just_completed_runners = q_runner.iter_many(&runner_just_completed);
    while let Some((runner_entity, runner, children)) = just_completed_runners.fetch_next() {
        if !runner.is_completed() {
            continue;
        }

        let children = children.iter().flat_map(|a| a.iter());
        let mut spans = q_span.iter_many_mut([&runner_entity].into_iter().chain(children));
        while let Some((span_entity, _, _)) = spans.fetch_next() {
            let Some(mut entity) = commands.get_entity(span_entity) else {
                continue;
            };
            entity.remove::<TimeSpanProgress>();
        }
    }
    runner_just_completed.clear();

    q_added_skip
        .iter()
        .for_each(|(runner_entity, _, children)| {
            let children = children.iter().flat_map(|a| a.iter());
            let mut spans = q_span.iter_many_mut([&runner_entity].into_iter().chain(children));
            while let Some((span_entity, _, _)) = spans.fetch_next() {
                let Some(mut entity) = commands.get_entity(span_entity) else {
                    continue;
                };
                entity.remove::<TimeSpanProgress>();
            }
        });

    q_runner
        .iter_mut()
        .for_each(|(runner_entity, mut runner, children)| {
            if runner.is_completed() {
                return;
            }

            let repeated =
                if runner.elasped().now_period.floor() as i32 != 0 && !runner.is_completed() {
                    runner.repeat.map(|r| r.1)
                } else {
                    None
                };

            let runner_elasped_now = runner.elasped().now;
            let runner_elasped_previous = runner.elasped().previous;
            let runner_direction = runner.direction;

            let children = children.iter().flat_map(|a| a.iter());
            let mut spans = q_span.iter_many_mut([&runner_entity].into_iter().chain(children));
            while let Some((span_entity, time_span_progress, span)) = spans.fetch_next() {
                let now_quotient = span.quotient(runner_elasped_now);
                let previous_quotient = span.quotient(runner_elasped_previous);

                let direction = if repeated.is_none() {
                    match runner_elasped_previous.total_cmp(&runner_elasped_now) {
                        Ordering::Less => TimeDirection::Forward,
                        Ordering::Equal => runner_direction,
                        Ordering::Greater => TimeDirection::Backward,
                    }
                } else {
                    runner_direction
                };

                let span_in_range =
                    span_in_range(direction, previous_quotient, now_quotient, repeated);

                if let Some(use_time) = span_in_range {
                    let span_max = span.max().duration().as_secs_f32();
                    let span_min = span.min().duration().as_secs_f32();

                    let span_length = span_max - span_min;

                    let new_now = match use_time {
                        UseTime::Current => runner_elasped_now - span_min,
                        UseTime::Min => 0.,
                        UseTime::Max => span_length,
                    };
                    let new_previous = runner_elasped_previous - span_min;

                    let new_now_percentage = if span_length > 0. {
                        new_now / span_length
                    } else {
                        match new_now.total_cmp(&0.) {
                            Ordering::Greater => f32::INFINITY,
                            Ordering::Equal => match runner_direction {
                                Forward => f32::INFINITY,
                                Backward => f32::NEG_INFINITY,
                            },
                            Ordering::Less => f32::NEG_INFINITY,
                        }
                    };
                    let new_previous_percentage = if span_length > 0. {
                        new_previous / span_length
                    } else {
                        match new_previous.total_cmp(&0.) {
                            Ordering::Greater => f32::INFINITY,
                            Ordering::Equal => match runner_direction {
                                Forward => f32::INFINITY,
                                Backward => f32::NEG_INFINITY,
                            },
                            Ordering::Less => f32::NEG_INFINITY,
                        }
                    };

                    match time_span_progress {
                        Some(mut time_span_progress) => {
                            time_span_progress.update(new_now, new_now_percentage);
                        }
                        None => {
                            commands.entity(span_entity).insert(TimeSpanProgress {
                                now_percentage: new_now_percentage,
                                now: new_now,
                                previous_percentage: new_previous_percentage,
                                previous: new_previous,
                            });
                        }
                    }
                } else {
                    commands.entity(span_entity).remove::<TimeSpanProgress>();
                }
            }
            runner.collaspe_elasped();
            if runner.is_completed() {
                runner_just_completed.push(runner_entity);
            }
        });

    enum UseTime {
        Current,
        Min,
        Max,
    }

    fn span_in_range(
        direction: TimeDirection,
        previous_quotient: DurationQuotient,
        now_quotient: DurationQuotient,
        repeated: Option<RepeatStyle>,
    ) -> Option<UseTime> {
        // Look at this behemoth of edge case handling.
        //
        // The edge cases are the time when the timer are really short
        // or delta is really long per frame.
        //
        // Currently unknown what happen when timer repeated multiple times in one frame.

        match (
            direction,
            previous_quotient,
            now_quotient,
            repeated,
        ) {
            (_, Inside, Inside, None) => {
                Some(UseTime::Current)
            },
            // ----------------------------------------------------------------
            | (Forward, Before, Inside, None)
            | (Forward, Inside, After, None)
            | (Forward, Before, After, None)
                => {
                    Some(UseTime::Current)
                },

            // ----------------------------------------------------------------
            | (Backward, After, Inside, None)
            | (Backward, Inside, Before, None)
            | (Backward, After, Before, None)
                => {
                    Some(UseTime::Current)
                },

            // -----------------------------------------------------------------
            // don't remove these comments, may use for debugging in the future
            | (Forward, Before, Before, Some(WrapAround)) // 1&2 max
            | (Forward, Inside, Before, Some(WrapAround)) // 1 max
                => {
                    // println!("forward wrap use max");
                    Some(UseTime::Max)
                },
            | (Forward, Before, Inside, Some(WrapAround)) // 2 now
            | (Forward, Before, After, Some(WrapAround)) // 2 now, max
            | (Forward, Inside, Inside, Some(WrapAround)) // 1&2 now
            | (Forward, Inside, After, Some(WrapAround)) // 2 now, max
            | (Forward, After, Inside, Some(WrapAround)) // 1 now 
            | (Forward, After, After, Some(WrapAround)) // 1&2 now, max
            // | (Forward, After, Before, Some(WrapAround)) // 1
                => {
                    // println!("forward wrap use current");
                    Some(UseTime::Current)
                },

            // ----------------------------------------------------------------
            | (Backward, After, After, Some(WrapAround)) // 1&2 min
            | (Backward, Inside, After, Some(WrapAround)) // 1 min
                => {
                    // println!("backward wrap use min");
                    Some(UseTime::Min)
                },
            | (Backward, Before, Before, Some(WrapAround)) // 1&2 now, min
            | (Backward, Before, Inside, Some(WrapAround)) // 1 now 
            | (Backward, Inside, Before, Some(WrapAround)) // 2 now, min
            | (Backward, Inside, Inside, Some(WrapAround)) // 1&2 now
            | (Backward, After, Before, Some(WrapAround)) // 2 now, min
            | (Backward, After, Inside, Some(WrapAround)) // 2 now
            // | (Backward, Before, After, Some(WrapAround)) // 1
                => {
                    // println!("backward wrap use current");
                    Some(UseTime::Current)
                },

            // ----------------------------------------------------------------
            | (Backward, Before, Before, Some(PingPong)) // 1&2 now, min
            | (Backward, Before, Inside, Some(PingPong)) // 1 now
            | (Backward, Before, After, Some(PingPong)) // 1 now, max
            | (Backward, Inside, Before, Some(PingPong)) // 2 now, min
            | (Backward, Inside, Inside, Some(PingPong)) // 1&2 now
            | (Backward, Inside, After, Some(PingPong)) // 1 now, max
            | (Backward, After, Before, Some(PingPong)) // 2 now, min
            | (Backward, After, Inside, Some(PingPong)) // 2 now
            // | (Backward, After, After, Some(PingPong)) // 1&2
                => Some(UseTime::Current),

            // ----------------------------------------------------------------
            // | (Forward, Before, Before, Some(PingPong)) // 1&2
            | (Forward, Before, Inside, Some(PingPong)) // 2 now
            | (Forward, Before, After, Some(PingPong)) // 2 now, max
            | (Forward, Inside, Before, Some(PingPong)) // 1 now, min
            | (Forward, Inside, Inside, Some(PingPong)) // 1&2 now
            | (Forward, Inside, After, Some(PingPong)) // 2 now, max
            | (Forward, After, Before, Some(PingPong)) // 1 now, min
            | (Forward, After, Inside, Some(PingPong)) // 1 now
            | (Forward, After, After, Some(PingPong)) // 1&2 now, max
                => Some(UseTime::Current),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use bevy_ecs::system::RunSystemOnce as _;

    use super::*;

    fn secs(secs: f32) -> Duration {
        Duration::from_secs_f32(secs)
    }

    // fn eq(lhs: f32, rhs: f32) -> bool {
    //     (lhs - rhs).abs() <= f32::EPSILON
    // }

    #[test]
    fn timer() {
        let mut timer = TimeRunner::new(secs(5.));

        timer.raw_tick(2.5);
        assert_eq!(timer.elasped.now, 2.5);
        assert_eq!(timer.elasped.now_period, 0.5);

        timer.raw_tick(2.5);
        assert_eq!(timer.elasped.now, 5.);
        assert_eq!(timer.elasped.now_period, 1.);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 5.);
        assert_eq!(timer.elasped.now_period, 1.);

        timer.set_tick(0.);

        timer.raw_tick(3.);
        assert_eq!(timer.elasped.now, 3.);
        assert_eq!(timer.elasped.now_period, 3. / 5.);

        timer.raw_tick(3.);
        assert_eq!(timer.elasped.now, 5.);
        assert_eq!(timer.elasped.now_period, 1.);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 5.);
        assert_eq!(timer.elasped.now_period, 1.);
    }

    #[test]
    fn timer_backward() {
        let mut timer = TimeRunner::new(secs(5.));
        timer.set_direction(TimeDirection::Backward);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 0.);
        assert_eq!(timer.elasped.now_period, 0.);

        timer.set_tick(5.);

        timer.raw_tick(2.5);
        assert_eq!(timer.elasped.now, 2.5);
        assert_eq!(timer.elasped.now_period, 0.5);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 1.5);
        assert_eq!(timer.elasped.now_period, 1.5 / 5.);

        timer.raw_tick(2.);
        assert_eq!(timer.elasped.now, 0.);
        assert_eq!(timer.elasped.now_period, 0.);
    }

    #[test]
    fn timer_wrap_around() {
        let mut timer = TimeRunner::new(secs(5.));
        timer.set_repeat(Some((Repeat::Infinitely, RepeatStyle::WrapAround)));

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 1.);
        assert_eq!(timer.elasped.now_period, 1. / 5.);

        timer.raw_tick(2.5);
        assert_eq!(timer.elasped.now, 3.5);
        assert_eq!(timer.elasped.now_period, 3.5 / 5.);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 4.5);
        assert_eq!(timer.elasped.now_period, 4.5 / 5.);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 0.5);
        assert_eq!(timer.elasped.now_period, 5.5 / 5.);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 1.5);
        assert_eq!(timer.elasped.now_period, 1.5 / 5.);

        timer.raw_tick(3.5);
        assert_eq!(timer.elasped.now, 0.);
        assert_eq!(timer.elasped.now_period, 5. / 5.);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 1.);
        assert_eq!(timer.elasped.now_period, 1. / 5.);
    }

    #[test]
    fn timer_backward_wrap_around() {
        let mut timer = TimeRunner::new(secs(5.));
        timer.set_repeat(Some((Repeat::Infinitely, RepeatStyle::WrapAround)));
        timer.set_direction(TimeDirection::Backward);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 4.);
        assert_eq!(timer.elasped.now_period, -1. / 5.);

        timer.raw_tick(2.5);
        assert_eq!(timer.elasped.now, 1.5);
        assert_eq!(timer.elasped.now_period, 1.5 / 5.);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 0.5);
        assert_eq!(timer.elasped.now_period, 0.5 / 5.);

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 4.5);
        assert_eq!(timer.elasped.now_period, -0.5 / 5.);
    }

    #[test]
    fn timer_wrap_around_times() {
        let mut timer = TimeRunner::new(secs(5.));
        timer.set_repeat(Some((Repeat::times(2), RepeatStyle::WrapAround)));

        timer.raw_tick(4.);
        assert_eq!(timer.elasped.now, 4.);
        assert_eq!(timer.elasped.now_period, 4. / 5.);
        assert_eq!(
            timer.repeat.unwrap().0,
            Repeat::Times {
                times: 2,
                times_repeated: 0
            },
        );

        timer.raw_tick(4.);
        assert_eq!(timer.elasped.now, 3.);
        assert_eq!(timer.elasped.now_period, 8. / 5.);
        assert_eq!(
            timer.repeat.unwrap().0,
            Repeat::Times {
                times: 2,
                times_repeated: 1
            },
        );

        timer.raw_tick(4.);
        assert_eq!(timer.elasped.now, 2.);
        assert_eq!(timer.elasped.now_period, 7. / 5.);
        assert_eq!(
            timer.repeat.unwrap().0,
            Repeat::Times {
                times: 2,
                times_repeated: 2
            },
        );

        timer.raw_tick(4.);
        assert_eq!(timer.elasped.now, 5.);
        assert_eq!(timer.elasped.now_period, 1.);
        assert_eq!(
            timer.repeat.unwrap().0,
            Repeat::Times {
                times: 2,
                times_repeated: 2
            },
        );

        timer.raw_tick(1.);
        assert_eq!(timer.elasped.now, 5.);
        assert_eq!(timer.elasped.now_period, 1.);
        assert_eq!(
            timer.repeat.unwrap().0,
            Repeat::Times {
                times: 2,
                times_repeated: 2
            },
        );
    }

    #[test]
    fn timer_backward_wrap_around_times() {
        let mut timer = TimeRunner::new(secs(5.));
        timer.set_repeat(Some((Repeat::times(2), RepeatStyle::WrapAround)));
        timer.set_direction(TimeDirection::Backward);

        timer.raw_tick(4.);
        assert_eq!(timer.elasped.now, 1.);
        assert_eq!(timer.elasped.now_period, -4. / 5.);
        assert_eq!(
            timer.repeat.unwrap().0,
            Repeat::Times {
                times: 2,
                times_repeated: 1
            },
        );

        timer.raw_tick(4.);
        assert_eq!(timer.elasped.now, 2.);
        assert_eq!(timer.elasped.now_period, -3. / 5.);
        assert_eq!(
            timer.repeat.unwrap().0,
            Repeat::Times {
                times: 2,
                times_repeated: 2
            },
        );

        timer.raw_tick(4.);
        assert_eq!(timer.elasped.now, 0.);
        assert_eq!(timer.elasped.now_period, 0. / 5.);
        assert_eq!(
            timer.repeat.unwrap().0,
            Repeat::Times {
                times: 2,
                times_repeated: 2
            },
        );
    }

    #[test]
    fn timer_ping_pong() {
        let mut timer = TimeRunner::new(secs(5.));
        timer.set_repeat(Some((Repeat::Infinitely, RepeatStyle::PingPong)));

        timer.raw_tick(3.);
        assert_eq!(timer.elasped.now, 3.);
        assert_eq!(timer.elasped.now_period, 3. / 5.);
        assert_eq!(timer.direction, TimeDirection::Forward);

        timer.raw_tick(3.);
        assert_eq!(timer.elasped.now, 4.);
        assert_eq!(timer.elasped.now_period, 6. / 5.);
        assert_eq!(timer.direction, TimeDirection::Backward);

        timer.raw_tick(3.);
        assert_eq!(timer.elasped.now, 1.);
        assert_eq!(timer.elasped.now_period, 1. / 5.);
        assert_eq!(timer.direction, TimeDirection::Backward);

        timer.raw_tick(3.);
        assert_eq!(timer.elasped.now, 2.);
        assert_eq!(timer.elasped.now_period, -2. / 5.);
        assert_eq!(timer.direction, TimeDirection::Forward);

        timer.raw_tick(3.);
        assert_eq!(timer.elasped.now, 5.);
        assert_eq!(timer.elasped.now_period, 5. / 5.);
        assert_eq!(timer.direction, TimeDirection::Backward);

        timer.raw_tick(3.);
        assert_eq!(timer.elasped.now, 2.);
        assert_eq!(timer.elasped.now_period, 2. / 5.);
        assert_eq!(timer.direction, TimeDirection::Backward);
    }

    // There's no test for repeating ones yet and I bet most of them is wrong.
    #[test]
    fn timer_big_tick() {
        let mut world = World::default();

        let mut time_runner = TimeRunner::new(secs(10.));
        time_runner.tick(10.);
        let mut time_span_id = Entity::PLACEHOLDER;
        world.spawn(time_runner).with_children(|c| {
            time_span_id = c
                .spawn(TimeSpan::try_from(secs(4.)..secs(6.)).unwrap())
                .id();
        });

        world.run_system_once(time_runner_system);

        let progress = world
            .entity(time_span_id)
            .get::<TimeSpanProgress>()
            .expect("TimeSpanProgress should be here");
        dbg!(progress);
        assert_eq!(
            *progress,
            TimeSpanProgress {
                now_percentage: 1.,
                now: 2.,
                previous_percentage: -2.,
                previous: -4.,
            }
        );
    }

    #[test]
    fn timer_zero_length_span() {
        let mut world = World::default();

        let mut time_runner = TimeRunner::new(secs(4.));
        time_runner.tick(4.);
        let mut time_span_id = Entity::PLACEHOLDER;
        world.spawn(time_runner).with_children(|c| {
            time_span_id = c
                .spawn(TimeSpan::try_from(secs(2.)..=secs(2.)).unwrap())
                .id();
        });

        world.run_system_once(time_runner_system);

        let progress = world
            .entity(time_span_id)
            .get::<TimeSpanProgress>()
            .expect("TimeSpanProgress should be here");
        dbg!(progress);
        assert_eq!(
            *progress,
            TimeSpanProgress {
                now_percentage: f32::INFINITY,
                now: 0.,
                previous_percentage: f32::NEG_INFINITY,
                previous: -2.,
            }
        );
    }
}
