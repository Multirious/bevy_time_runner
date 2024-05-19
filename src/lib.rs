//! # bevy_time_runner
//!
//! # [`TimeRunner`]
//! [`TimeRunner`] will expects you to create an entity containing the [`TimeRunner`]
//! component and their children to contain the [`TimeSpan`] component like so:
//!
//! ```
//! # use bevy::ecs::system::CommandQueue;
//! # use bevy::ecs::system::Commands;
//! # use bevy::prelude::*;
//! # use std::time::Duration;
//! #
//! fn secs(secs: u64) -> Duration {
//!     Duration::from_secs(secs)
//! }
//!
//! # let world = World::default();
//! # let mut queue = CommandQueue::default();
//! # let mut commands = Commands::new(&mut queue, &world);
//! #
//! commands
//!     .spawn(TimeRunner::new(secs(10)))
//!     .with_children(|c| {
//!         c.spawn(TimeSpan::try_from(secs(0)..secs(3)).unwrap());
//!         c.spawn(TimeSpan::try_from(secs(3)..secs(7)).unwrap());
//!         c.spawn(TimeSpan::try_from(secs(7)..secs(10)).unwrap());
//!     });
//! ```
//! While the [`TimeRunner`] is running, for each child with the component [`TimeSpan`],
//! the component [`TimeSpanProgress`] will be inserted to each child with the appropriate
//! values and removed if the runner is out of range of the span.
//!
//! This creates a very flexible timing system that's useful for variety of purposes.
//!
#![warn(missing_docs)]

use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::prelude::*;

mod time_runner;
mod time_span;
pub use time_runner::*;
pub use time_span::*;

/// Add [`time_runner_system`]
/// Registers [`TimeRunner`]
#[derive(Debug)]
pub struct TimeRunnerPlugin {
    /// All systems will be put to this schedule
    pub schedule: InternedScheduleLabel,
}

impl Default for TimeRunnerPlugin {
    fn default() -> Self {
        TimeRunnerPlugin {
            schedule: PostUpdate.intern(),
        }
    }
}

impl Plugin for TimeRunnerPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "bevy_eventlistener")]
        app.add_plugins(bevy_eventlistener::EventListenerPlugin::<TimeRunnerEnded>::default());

        app.configure_sets(
            self.schedule,
            (TimeRunnerSet::TickTimer, TimeRunnerSet::Progress).chain(),
        )
        .add_systems(
            self.schedule,
            (
                tick_time_runner_system.in_set(TimeRunnerSet::TickTimer),
                time_runner_system.in_set(TimeRunnerSet::Progress),
            ),
        )
        .add_event::<TimeRunnerEnded>()
        .register_type::<TimeRunner>();
    }
}

/// System set in this crate
#[derive(Debug, PartialEq, Eq, Hash, Clone, SystemSet)]
pub enum TimeRunnerSet {
    /// Systems responsible for ticking timer
    TickTimer,
    /// Systems responsible for updating [`TimeSpanProgress`]
    Progress,
}
