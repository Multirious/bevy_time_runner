//! # bevy_time_runner
//! General timing system

#![warn(missing_docs)]

use bevy::{ecs::schedule::InternedScheduleLabel, prelude::*};

mod time_runner;
mod time_span;
pub use time_runner::*;
pub use time_span::*;

/// Add [`time_runner_system`]
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
