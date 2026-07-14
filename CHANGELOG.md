# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Logical/physical pixel conversion and window position/size setting bugs (#3).

### Changed

- Migrated FFI implementation to the objc2 crate family (#2).
- Removed `WindowError::ArbitraryError` in favor of `WindowError::OsError` (#5).
- Use stable toolchain (#3).

## [0.1.1] - 2026-07-13

### Fixed

- [docs.rs](https://docs.rs/fowin/latest/fowin/) build (#6).

## [0.1.0] - 2025-05-21

### Added

- Everything

[unreleased]: https://github.com/ok-nick/fowin/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/ok-nick/fowin/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ok-nick/fowin/releases/tag/v0.1.0
