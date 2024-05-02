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

use bevy::ecs::schedule::InternedScheduleLabel;
use bevy::prelude::*;

mod time_runner;
mod time_span;
mod time_span_group;
pub use time_runner::*;
pub use time_span::*;
pub use time_span_group::*;

/// Add [`time_runner_system`]
/// Registers [`TimeRunner`]
#[derive(Debug)]
pub struct TimeRunnerPlugin {
    /// The schedule this plugin will use
    pub schedule: InternedScheduleLabel,
}

impl Plugin for TimeRunnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(self.schedule, time_runner_system)
            .register_type::<TimeRunner>();
    }
}
