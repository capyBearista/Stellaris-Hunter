# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-06-27

### Added
- **Tauri v2 Desktop App**: Initial release of the Stellaris Hunter desktop application for Windows.
- **React + TypeScript Frontend**: A responsive, read-only dashboard shell built with Vite 8.
- **Local Discovery**: Automatic detection of Stellaris installation and Documents folders.
- **Save Parsing**: In-memory parsing of Stellaris save files (ZIP + Clausewitz format) to extract game state, empire setup, and progression facts.
- **Launcher Parsing**: Read-only parsing of `launcher-v2.sqlite` to detect active playsets and enabled mods.
- **Achievement Catalog**: Bundled curated catalog (v1.1.0) covering all 211 Stellaris achievements with conditions, tags, and hints.
- **Catalog Sync**: Ability to sync the latest achievement catalog directly from the GitHub repository.
- **Steam Integration**: Read-only Steam achievement sync to establish the player's baseline completion state.
- **Icon Cache**: Automatic fetching and local caching of achievement icons from Steam.
- **Rule Engine**: Conservative evaluation engine that compares parsed save facts against catalog conditions to determine achievement statuses (Completed, Planned, Possible, Incompatible, Impossible, Unknown).
- **Planner UI**: Dedicated view to plan achievements for a specific run, grouped by computed status.
- **Manual Overrides**: Ability to manually override achievement completion status and individual run facts.
- **Notes System**: Support for adding custom notes to runs and specific achievements within a run.
- **Eligibility Detection**: Conservative analysis of save files and active mods to determine if a run is likely eligible for achievements.
- **UX Polish**: 
  - Overview dashboard with aggregate planner status counts and Steam sync status.
  - Deep-linking from the Runs page directly to the Planner for a specific run.
  - Explicit save eligibility panel on the Runs page.
  - "Parse Latest Save" action for individual runs without requiring a full Documents rescan.

### Security
- **Read-Only Constraint**: Strict enforcement that the app never mutates external Stellaris, Steam, or Documents files.
- **Steam API Quarantine**: Mutating Steam APIs are explicitly forbidden and enforced via CI tests.
