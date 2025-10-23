# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.13.1](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.13.0...fluent-templates-v0.13.1) - 2025-08-25

### Other

- improve the error messages for unknown localizations ([#94](https://github.com/XAMPPRocky/fluent-templates/pull/94))
- Bump Fluent crates to latest minor releases ([#92](https://github.com/XAMPPRocky/fluent-templates/pull/92))
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.13.2](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.13.1...fluent-templates-v0.13.2) - 2025-10-23

### Other

- update thiserror to 2.0 ([#97](https://github.com/XAMPPRocky/fluent-templates/pull/97))
- make MultiLoader useable in statics ([#96](https://github.com/XAMPPRocky/fluent-templates/pull/96))

## [0.13.0](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.12.1...fluent-templates-v0.13.0) - 2025-01-12

### Added

- [**breaking**] Replace once_cell Lazy with standard lib LazyLock (#86)

## [0.12.1](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.12.0...fluent-templates-v0.12.1) - 2025-01-09

### Other

- Add a `MultiLoader` loader that consists of other loaders ([#84](https://github.com/XAMPPRocky/fluent-templates/pull/84))

## [0.12.0](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.11.0...fluent-templates-v0.12.0) - 2024-09-27

### Fixed

- [**breaking**] Make `Loader` object-safe ([#83](https://github.com/XAMPPRocky/fluent-templates/pull/83))

## [0.11.0](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.10.1...fluent-templates-v0.11.0) - 2024-09-22

### Added

- [**breaking**] upgrade to handlebars 6 ([#79](https://github.com/XAMPPRocky/fluent-templates/pull/79))

## [0.10.1](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.10.0...fluent-templates-v0.10.1) - 2024-08-19

### Fixed
- Drop `fluent` dependency ([#76](https://github.com/XAMPPRocky/fluent-templates/pull/76))

### Other
- Optimize fallback initialization on static loader macro ([#77](https://github.com/XAMPPRocky/fluent-templates/pull/77))

## [0.10.0](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.9.4...fluent-templates-v0.10.0) - 2024-08-16

### Added
- [**breaking**] Relax dependencies ([#74](https://github.com/XAMPPRocky/fluent-templates/pull/74))
- swap panic for error handling ([#63](https://github.com/XAMPPRocky/fluent-templates/pull/63))

### Other
- Make `flume` dependency optional and cleanup features ([#73](https://github.com/XAMPPRocky/fluent-templates/pull/73))
- Make `fallback` fields of `StaticLoader` and `ArcLoader` public ([#71](https://github.com/XAMPPRocky/fluent-templates/pull/71))

## [0.9.4](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.9.3...fluent-templates-v0.9.4) - 2024-04-22

### Other
- Bump `heck` to `v0.5.0` ([#67](https://github.com/XAMPPRocky/fluent-templates/pull/67))

## [0.9.3](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.9.2...fluent-templates-v0.9.3) - 2024-04-17

### Other
- Move common dependencies up to workspace level ([#64](https://github.com/XAMPPRocky/fluent-templates/pull/64))

## [0.9.2](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.9.1...fluent-templates-v0.9.2) - 2024-04-12

### Other
- Add follow_links to support builds with Bazel ([#62](https://github.com/XAMPPRocky/fluent-templates/pull/62))
- remove snafu ([#60](https://github.com/XAMPPRocky/fluent-templates/pull/60))

- Remove unused `lazy_static`, replace `snafu` with `thiserror`, bump `flume` to `0.11`

## [0.9.1](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.9.0...fluent-templates-v0.9.1) - 2024-03-10

### Other
- Check that `fallback_language` exists between locales building static bundles ([#58](https://github.com/XAMPPRocky/fluent-templates/pull/58))

## [0.9.0](https://github.com/XAMPPRocky/fluent-templates/compare/fluent-templates-v0.8.0...fluent-templates-v0.9.0) - 2024-03-04

### Fixed
- fix typo
- fixes [#10](https://github.com/XAMPPRocky/fluent-templates/pull/10) -- port info on templating language features from README to documentation ([#11](https://github.com/XAMPPRocky/fluent-templates/pull/11))
- fix reade
- fix missing link

### Other
- disable tera features
- Use resolve 2
- Use workspace.package
- Update release-plz.yaml
- Update to support handlebars v5 ([#55](https://github.com/XAMPPRocky/fluent-templates/pull/55))
- Create release-plz.yaml
- Sort the 'insert_resources' data to make static_loader! deterministic ([#56](https://github.com/XAMPPRocky/fluent-templates/pull/56))
- Bump syn to 2.0 ([#50](https://github.com/XAMPPRocky/fluent-templates/pull/50))
- Add ignore as optional feature ([#48](https://github.com/XAMPPRocky/fluent-templates/pull/48))
- Reverts changes from PR [#33](https://github.com/XAMPPRocky/fluent-templates/pull/33) and adds them with a "try_" prefix. ([#47](https://github.com/XAMPPRocky/fluent-templates/pull/47))
- Accept FnMut instead of fn pointer in ArcLoaderBuilder::customize ([#40](https://github.com/XAMPPRocky/fluent-templates/pull/40))
- Release 0.8.0
- Change the lookup function to be a better fallback chain ([#35](https://github.com/XAMPPRocky/fluent-templates/pull/35))
- Remove flume default features ([#36](https://github.com/XAMPPRocky/fluent-templates/pull/36))
- :lookup/lookup_complete/lookup_with_args return Option<String> ([#33](https://github.com/XAMPPRocky/fluent-templates/pull/33))
- Update rust.yml
- Release 0.7.1
- Release 0.7.0
- Update proc macro deps
- Update templating deps
- Update deps and file layout
- Release 0.6.1
- Updated fluent dependency to 0.14 ([#26](https://github.com/XAMPPRocky/fluent-templates/pull/26))
- Release 0.6.0
- Added attribute lookup support and refactored lookups to share code
- Borrowed argument keys support. ([#25](https://github.com/XAMPPRocky/fluent-templates/pull/25))
- add js feature to getrandom
- Update dependencies
- cfg out tempfile for wasm dev-dependencies
- Update README.md
- Release 0.5.16
- Point to readme in toml
- Release 0.5.15
- remove dead code
- Release 0.5.14
- re-export language ident and hide new method
- Release 0.5.13
- Add function that returns available locales ([#17](https://github.com/XAMPPRocky/fluent-templates/pull/17))
- Fix compile error with flume 0.7.2 ([#18](https://github.com/XAMPPRocky/fluent-templates/pull/18))
- Release 0.5.12
- Fallback to default when requested language is missing, rather than panicking ([#16](https://github.com/XAMPPRocky/fluent-templates/pull/16))
- Release 0.5.11
- Default lang argument in Tera templates ([#14](https://github.com/XAMPPRocky/fluent-templates/pull/14))
- Release 0.5.10
- Return errors from ArcLoaderBuilder.build(). ([#9](https://github.com/XAMPPRocky/fluent-templates/pull/9))
- Release 0.5.9
- Change core resource to use include_str
- Release 0.5.8
- Use include_str instead of reading file
- Update README.md ([#7](https://github.com/XAMPPRocky/fluent-templates/pull/7))
- Release 0.5.7
- minor nit
- Release 0.5.6
- Release 0.5.5
- Update toml
- Adding CARGO_MANIFEST_DIR to make paths consistent regardless of crate location within a workspace. ([#6](https://github.com/XAMPPRocky/fluent-templates/pull/6))
- Add proper message when failing to read locales
- Update README.md
- Update rust.yml
- Update rust.yml
- Update rust.yml
- Update rust.yml
- Update rust.yml
- Update rust.yml
- Update rust.yml
- Delete .travis.yml
- Update README
- Release 0.5.4
- Add visibility support for static loader
- Update README.md
- Update README.md
- Update README.md
- Update README.md
- Update README.md
- Update README.md
- Update README.md
- Update rust.yml
- Update rust.yml
- Update rust.yml
- Update rust.yml
- Update rust.yml
- Update CI
- Release 0.5.3
- Include macro by default
- Release 0.5.2
- Add docs.rs metadata
- Update README.md
- Update README.md
- Update rust.yml
- Release 0.5.1
- Update rust.yml
- fmt
- Update toml
- set version in toml
- Release 0.5.0
- Update README.md
- polish API
- Update documentation
- Fixed missing field not being handled
- fmt
- Renamed FluentHelper to FluentLoader
- Move static_loader to procedural macro
- Move to cargo workspace
- Move to using ignore for reading directories.
- Bump version
- Fixed multi word parametrs and changed args to take correct hashmap type.
- bump version
- Update dependencies
- Bump version
- Update top level documentation
- Add tera support
- Removed dbg statement
- Document everything and split handlebars into its own feature
- Format code
- Rename crate
- Remove unused imports
- Remove old read_from_file
- Switch to LoaderError as default error type
- Update github actions CI config
- Refactor fs into its own modules and add error type
- Version bump
- Update types and test code
- Version bump
- Update core to shared_resources and allow multiple files, and refactor out some unwraps
- Bump version
- Remove debug
- Version bump
- Switch from strs to paths
- Version bump
- Update description
- Set version to 0.4.0
- Update Handlebars
- Rename crate
- Add arc loader and refactor tests
- Update rust.yml
- Update rust.yml
- Update rust.yml
- Update README.md
- Update README.md
- Update rust.yml
- Update README.md
- Create rust.yml
- Make package template engine agnostic and add Tera support
- Update FluentBundle to use concurrent variant
- Bump to 0.3.0
- Merge pull request [#4](https://github.com/XAMPPRocky/fluent-templates/pull/4) from XAMPPRocky/upgrade-fluent
- Upgrade Fluent
- only load .ftl files as fluent files
- Release 0.2.0
- Mention constructor cost
- Macro fixes
- Add main docs
- Add documentation for loader
- Add customizer
- Clarify that this helper is for the Rust implementation, link JS
- Fill in tests
- test skeleton
- Add travis
- Fixes
- Move from identifier to string
- Remove incontext support
- text -> fluent
- Turn loader into a macro
- I18NHelper -> FluentHelper
- Turn loader into trait
- Split out loader
- Import I18nHelper from www.rust-lang.org
- Initial commit
