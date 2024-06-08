use std::cmp::Ordering;
use std::ops;
use std::time::Duration;

use bevy_ecs::prelude::*;
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;

/// Bounding enum for [`Duration`] to be exclusivively checked or inclusivively
/// checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum TimeBound {
    /// Inclusively check this duration
    Inclusive(Duration),
    /// Exclusively check this duration
    Exclusive(Duration),
}

impl TimeBound {
    /// Get the inner duration
    pub fn duration(&self) -> Duration {
        match self {
            TimeBound::Inclusive(d) | TimeBound::Exclusive(d) => *d,
        }
    }
}

impl Default for TimeBound {
    fn default() -> Self {
        TimeBound::Inclusive(Duration::ZERO)
    }
}

/// Error type for when creating a new [`TimeSpan`].
#[derive(Debug)]
pub enum NewTimeSpanError {
    /// The provided min, max will result in a [`TimeSpan`] that does not
    /// appear on a timeline
    NotTime {
        #[allow(missing_docs)]
        min: TimeBound,
        #[allow(missing_docs)]
        max: TimeBound,
    },
    /// The provided min is greater than max and it's not allowed.
    MinGreaterThanMax {
        #[allow(missing_docs)]
        min: TimeBound,
        #[allow(missing_docs)]
        max: TimeBound,
    },
}

impl std::error::Error for NewTimeSpanError {}
impl std::fmt::Display for NewTimeSpanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewTimeSpanError::NotTime { min, max } => {
                write!(
                    f,
                    "This span does not contain any time: min {min:?} max {max:?}"
                )
            }
            NewTimeSpanError::MinGreaterThanMax { min, max } => {
                write!(
                    f,
                    "This span has min greater than max: min {min:?} max {max:?}"
                )
            }
        }
    }
}

/// Define the range of time
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
pub struct TimeSpan {
    /// Minimum time of this time span.
    min: TimeBound,
    /// Maximum time of this time span.
    max: TimeBound,
}
impl TimeSpan {
    /// Create a new [`TimeSpan`] unchecked for invalid min, max.
    pub(crate) fn new_unchecked(min: TimeBound, max: TimeBound) -> TimeSpan {
        TimeSpan { min, max }
    }

    /// Create a new [`TimeSpan`]
    pub fn new(min: TimeBound, max: TimeBound) -> Result<TimeSpan, NewTimeSpanError> {
        if matches!(
            (min, max),
            (TimeBound::Exclusive(_), TimeBound::Exclusive(_))
        ) && min.duration() == max.duration()
        {
            return Err(NewTimeSpanError::NotTime { min, max });
        } else if min.duration() > max.duration() {
            return Err(NewTimeSpanError::MinGreaterThanMax { min, max });
        }
        Ok(Self::new_unchecked(min, max))
    }

    pub(crate) fn quotient(&self, secs: f32) -> DurationQuotient {
        let after_min = match self.min {
            TimeBound::Inclusive(min) => secs >= min.as_secs_f32(),
            TimeBound::Exclusive(min) => secs > min.as_secs_f32(),
        };
        let before_max = match self.max {
            TimeBound::Inclusive(max) => secs <= max.as_secs_f32(),
            TimeBound::Exclusive(max) => secs < max.as_secs_f32(),
        };
        match (after_min, before_max) {
            (true, true) => DurationQuotient::Inside,
            (true, false) => DurationQuotient::After,
            (false, true) => DurationQuotient::Before,
            (false, false) => unreachable!(),
        }
    }

    /// Get the min time
    pub fn min(&self) -> TimeBound {
        self.min
    }

    /// Get the max time
    pub fn max(&self) -> TimeBound {
        self.max
    }

    /// `self.max.duration() - self.min.duration()`
    pub fn length(&self) -> Duration {
        self.max.duration() - self.min.duration()
    }
}

impl Default for TimeSpan {
    fn default() -> Self {
        TimeSpan::try_from(Duration::ZERO..Duration::ZERO).unwrap()
    }
}

impl TryFrom<ops::Range<Duration>> for TimeSpan {
    type Error = NewTimeSpanError;

    fn try_from(range: ops::Range<Duration>) -> Result<Self, Self::Error> {
        TimeSpan::new(
            TimeBound::Inclusive(range.start),
            TimeBound::Exclusive(range.end),
        )
    }
}
impl TryFrom<ops::RangeInclusive<Duration>> for TimeSpan {
    type Error = NewTimeSpanError;

    fn try_from(range: ops::RangeInclusive<Duration>) -> Result<Self, Self::Error> {
        TimeSpan::new(
            TimeBound::Inclusive(*range.start()),
            TimeBound::Inclusive(*range.end()),
        )
    }
}

impl TryFrom<ops::RangeTo<Duration>> for TimeSpan {
    type Error = NewTimeSpanError;

    fn try_from(range: ops::RangeTo<Duration>) -> Result<Self, Self::Error> {
        TimeSpan::new(
            TimeBound::Inclusive(Duration::ZERO),
            TimeBound::Exclusive(range.end),
        )
    }
}

impl TryFrom<ops::RangeToInclusive<Duration>> for TimeSpan {
    type Error = NewTimeSpanError;

    fn try_from(range: ops::RangeToInclusive<Duration>) -> Result<Self, Self::Error> {
        TimeSpan::new(
            TimeBound::Inclusive(Duration::ZERO),
            TimeBound::Inclusive(range.end),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DurationQuotient {
    Before,
    Inside,
    After,
}

/// [`TimeSpanProgress`] is automatically managed by its runner.
#[derive(Debug, Default, Clone, Copy, PartialEq, Component)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
pub struct TimeSpanProgress {
    /// Value between 0–1 signalling the progress in percentage.
    /// Value can be more than 1 or negative to account for overshooting
    /// and undershooting. It's up to the implementor on how to deal with this.
    pub now_percentage: f32,
    /// Now in seconds that should be relative to the current span
    pub now: f32,
    /// Value between 0–1 signalling the progress of in percentage.
    /// Value can be more than 1 or negative to account for overshooting
    /// and undershooting. It's up to the implementor on how to deal with this.
    pub previous_percentage: f32,
    /// Previous in seconds that should be relative to the current span
    pub previous: f32,
}

impl TimeSpanProgress {
    /// Direction of the progress
    pub fn direction(&self) -> Option<TimeDirection> {
        match self.now.total_cmp(&self.previous) {
            Ordering::Greater => Some(TimeDirection::Forward),
            Ordering::Less => Some(TimeDirection::Backward),
            Ordering::Equal => None,
        }
    }

    pub(crate) fn update(&mut self, now: f32, now_percentage: f32) {
        self.previous_percentage = self.now_percentage;
        self.previous = self.now;
        self.now_percentage = now_percentage;
        self.now = now;
    }
}

/// Time direciton
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum TimeDirection {
    #[default]
    #[allow(missing_docs)]
    Forward,
    #[allow(missing_docs)]
    Backward,
}
