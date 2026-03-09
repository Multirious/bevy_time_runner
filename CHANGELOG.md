# Changelog

## Unreleased - XXXX-XX-XX
- Technically breaking (But hidden behind plugin):
  - Add `enable_debug` field to `TimeRunnerPlugin` by [#15](https://github.com/Multirious/bevy_time_runner/pull/15)
  - Add `Tagging` variant to `TimeRunnerSet` by [#15](https://github.com/Multirious/bevy_time_runner/pull/15)
  - Systems now expected `TimeCtx` generic parameter by [#15](https://github.com/Multirious/bevy_time_runner/pull/15)
  - `TimeRunnerPlugin` now expected `TimeCtx` generic parameter by [#19](https://github.com/Multirious/bevy_time_runner/pull/19)

- Systems can now be registered in non-default time context. (Virtual, Fixed, and/or Real) See [#15](https://github.com/Multirious/bevy_time_runner/pull/15) for more details.

  Notably:
  - `TimeRunnerPlugin` which now uses `TimeRunnerSystemsPlugin<()>` by default.
  - `TimeRunner` is always expected to have the `TimeContext<TimeCtx>` marker component.
  - `TimeContext<TimeCtx>` is automatically inserted to children of `TimeRunner`.
  - Add feature `debug`. This adds `TimeRunnerDebugPlugin` which logs warnings on missing `TimeContext<TimeCtx>` marker component when enabled.

- Migrate to bevy 0.18 by [#16](https://github.com/Multirious/bevy_time_runner/pull/16)

Internal:
- Update issue and PR template by [#21](https://github.com/Multirious/bevy_time_runner/pull/21)
- Update flake by [#17](https://github.com/Multirious/bevy_time_runner/pull/17)
  - Use latest instead of a version for stableRust in flake.nix
  - `nix flake update`
  - Remove flake-utils dependency from flake.nix

## v0.5.2 - 2025-10-6
- Fix documentation

## v0.5.1 - 2025-10-03
- Update Bevy to 0.17.2

## v0.5.0 - 2025-10-03
- Migrate to bevy 0.17

## v0.4.0 - 2025-05-09
- Migrate to bevy 0.16
- Change Rust edition to 2024
- Add Nix flake

## v0.3.0 - 2024-12-09

- Migrate to bevy 0.15
- Observer support [#4](https://github.com/Multirious/bevy_time_runner/pull/4)
- Fix ticking system using the wrong `Time` resource [#6](https://github.com/Multirious/bevy_time_runner/pull/6)
- Update nightly doc.rs build script [#7](https://github.com/Multirious/bevy_time_runner/pull/7)
- Fix unused imports when not using the `bev_app` feature [#8](https://github.com/Multirious/bevy_time_runner/pull/8)
- Update tests

## v0.2.0 - 2024-07-05

- Migrate `bevy` to 0.14
