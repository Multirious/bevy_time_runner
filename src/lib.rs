//! # bevy_time_runner
//!
//! # [`TimeRunner`]
//! [`TimeRunner`] will expects you to create an entity containing the [`TimeRunner`]
//! component and their children to contain the [`TimeSpan`] component like so:
//!
//! ```
//! # use bevy::ecs::world::CommandQueue;
//! # use bevy::ecs::system::Commands;
//! # use std::time::Duration;
//! use bevy::prelude::*;
//! use bevy_time_runner::{TimeRunner, TimeSpan};
//!
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
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]

#[cfg(feature = "bevy_app")]
use bevy_app::prelude::*;
#[cfg(feature = "debug")]
use bevy_ecs::component::ComponentId;
use bevy_ecs::prelude::*;
#[cfg(feature = "bevy_app")]
use bevy_ecs::schedule::{InternedScheduleLabel, ScheduleLabel};

mod time_runner;
mod time_span;
#[cfg(feature = "debug")]
use std::any::TypeId;
#[cfg(feature = "bevy_app")]
use std::marker::PhantomData;
pub use time_runner::*;
pub use time_span::*;

/// Add [`time_runner_system::<TimeCtx>`] on schedule
#[cfg(feature = "bevy_app")]
#[derive(Debug)]
pub struct TimeRunnerSystemsPlugin<TimeCtx = ()>
where
    TimeCtx: Default + Send + Sync + 'static,
{
    /// All systems will be put to this schedule
    pub schedule: InternedScheduleLabel,
    /// The time step ticked by (for example, () for regular time or Fixed for fixed time steps)
    _time_step: PhantomData<TimeCtx>,
}

#[cfg(feature = "bevy_app")]
impl<TimeCtx> TimeRunnerSystemsPlugin<TimeCtx>
where
    TimeCtx: Default + Send + Sync + 'static,
{
    /// Initializes the plugin to run on the specified schedule
    pub fn from_schedule_intern(schedule: InternedScheduleLabel) -> Self {
        Self {
            schedule,
            _time_step: Default::default(),
        }
    }
}

#[cfg(feature = "bevy_app")]
/// Registers all types and adds TimeRunnerRegistrationPlugin with default config
pub struct TimeRunnerPlugin {
    /// The schedule where the default time runners will be registered (TimerRunner<()>)
    pub schedule: InternedScheduleLabel,
    /// Enables [`TimeRunnerDebugPlugin`] with default configuration.
    /// You may manually insert [`TimeRunnerDebugPlugin`] for custom configuration.
    #[cfg(feature = "debug")]
    pub enable_debug: bool,
}

#[cfg(feature = "bevy_app")]
impl Default for TimeRunnerPlugin {
    fn default() -> Self {
        TimeRunnerPlugin {
            schedule: PostUpdate.intern(),
            #[cfg(feature = "debug")]
            enable_debug: true,
        }
    }
}

#[cfg(feature = "bevy_app")]
impl Plugin for TimeRunnerPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TimeRunnerSystemsPlugin<()>>() {
            app.add_plugins(TimeRunnerSystemsPlugin::<()>::from_schedule_intern(
                self.schedule,
            ));
        }
        app.add_message::<TimeRunnerEnded>();

        #[cfg(feature = "bevy_reflect")]
        app.register_type::<TimeRunner>()
            .register_type::<SkipTimeRunner>()
            .register_type::<TimeRunnerElasped>()
            .register_type::<TimeRunnerEnded>()
            .register_type::<TimeSpan>()
            .register_type::<TimeSpanProgress>()
            .register_type::<Repeat>()
            .register_type::<RepeatStyle>()
            .register_type::<TimeBound>()
            .register_type::<TimeDirection>();

        #[cfg(feature = "debug")]
        if self.enable_debug && !app.is_plugin_added::<TimeRunnerDebugPlugin>() {
            app.add_plugins(TimeRunnerDebugPlugin::default());
        }
    }
}

#[cfg(feature = "bevy_app")]
impl<TimeCtx> Plugin for TimeRunnerSystemsPlugin<TimeCtx>
where
    TimeCtx: Default + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.configure_sets(
            self.schedule,
            (TimeRunnerSet::TickTimer, TimeRunnerSet::Progress).chain(),
        )
        .add_systems(
            self.schedule,
            (
                tag_time_runner_children_with_context::<TimeCtx>.in_set(TimeRunnerSet::Tagging),
                tick_time_runner_system::<TimeCtx>.in_set(TimeRunnerSet::TickTimer),
                time_runner_system::<TimeCtx>.in_set(TimeRunnerSet::Progress),
            ),
        );
    }
}

/// System set in this crate
#[derive(Debug, PartialEq, Eq, Hash, Clone, SystemSet)]
pub enum TimeRunnerSet {
    /// Systems responsible for automatic tagging of time runner children
    Tagging,
    /// Systems responsible for ticking timer
    TickTimer,
    /// Systems responsible for updating [`TimeSpanProgress`]
    Progress,
}

/// Includes infos at runtime for debugging TimeRunner related issues.
/// This is inserted via [`TimeRunnerDebugPlugin`].
#[cfg(feature = "debug")]
#[derive(Debug, Default, Clone, Resource)]
pub struct TimeRunnerDebugInfo {
    time_steps: Vec<ComponentId>,
}

/// Debugs TimeRunner related issues
///
/// By default, this print warnings for missing [`TimeContext<T>`] where `T` is:
/// - `()` (The default time step),
/// - [`bevy_time::Fixed`],
/// - [`bevy_time::Real`] and
/// - [`bevy_time::Virtual`].
///
/// If you have addtional custom context, you may add it via [`Self::add_time_step`].
#[cfg(feature = "debug")]
pub struct TimeRunnerDebugPlugin {
    time_step_markers: Vec<(TypeId, &'static str)>,
}

#[cfg(feature = "debug")]
impl Default for TimeRunnerDebugPlugin {
    fn default() -> Self {
        let mut a = TimeRunnerDebugPlugin {
            time_step_markers: Vec::new(),
        };
        a.add_time_step::<()>();
        a.add_time_step::<bevy_time::Fixed>();
        a.add_time_step::<bevy_time::Real>();
        a.add_time_step::<bevy_time::Virtual>();
        a
    }
}

#[cfg(feature = "debug")]
impl TimeRunnerDebugPlugin {
    /// Enables a [`bevy_time::Time`]'s specific context to be checked.
    pub fn add_time_step<TimeCtx>(&mut self)
    where
        TimeCtx: Default + Send + Sync + 'static,
    {
        self.time_step_markers.push((
            TypeId::of::<TimeContext<TimeCtx>>(),
            std::any::type_name::<TimeContext<TimeCtx>>(),
        ));
    }
}

#[cfg(all(feature = "debug", feature = "bevy_app"))]
impl Plugin for TimeRunnerDebugPlugin {
    /// # Panics
    ///
    /// This method panics if a requested component to be debug haven't been registered.
    fn build(&self, app: &mut App) {
        let mut info = TimeRunnerDebugInfo::default();
        let world_mut = app.world_mut();
        world_mut.register_component::<TimeContext<()>>();
        world_mut.register_component::<TimeContext<bevy_time::Fixed>>();
        world_mut.register_component::<TimeContext<bevy_time::Real>>();
        world_mut.register_component::<TimeContext<bevy_time::Virtual>>();
        for (type_id, type_name) in &self.time_step_markers {
            let Some(component_id) = app.world().components().get_id(*type_id) else {
                panic!(
                    "{type_name} have not been registered as a componenet yet. It is required for `TimeRunnerDebugPlugin`."
                )
            };
            info.time_steps.push(component_id);
        }
        app.world_mut().insert_resource(info);
    }
}
