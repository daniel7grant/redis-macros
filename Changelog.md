# Changelog

## [Unreleased]

## [0.5.6] - 2025-07-20

### Added

- Add support for multiple generics (by @siennathesane)

### Updated

- Update dependencies

## [0.5.5] - 2025-06-10

### Updated

- Update dependencies to support Redis 0.32
- Update deadpool-redis to 0.21

## [0.5.4] - 2025-05-16

### Updated

- Update dependencies to support Redis 0.31

## [0.5.3] - 2025-05-06

### Updated

- Update dependencies to support Redis 0.30

## [0.5.2] - 2025-03-02

### Updated

- Update deadpool-redis and move to dev dependencies (by @negezor)

## [0.5.1] - 2025-02-18

### Updated

- Update dependencies to support Redis 0.29

## [0.5.0] - 2025-01-21

### Updated

- Update dependencies to support Redis 0.28
- Fix examples

## [0.4.3] - 2024-12-27

### Updated

- Update dependencies to fix issue with punycode parsing in idna

## [0.4.2] - 2024-09-15

### Updated

- Change dependency version to be fixed for minor versions

## [0.4.1] - 2024-09-13

### Updated

- Update redis to 0.27
- Update deadpool-redis to 0.17

## [0.4.0] - 2024-07-29

### Updated

- Update for new redis-rs (by @kristoferb)
    - Update redis to 0.26
    - Update deadpool-redis to 0.16
- Update versions on other crates

## [0.3.0] - 2024-04-01

### Updated

- Update dependencies (by @negezor)

## [0.2.1] - 2023-07-05

### Added

- Support for generic structures

## [0.2.0] - 2023-07-02

### Changed

- Remove absolute references from macros, so it works with reexporting crates

## [0.1.1] - 2023-02-05

### Added

- Unit testing
- Feature testing and documentation with examples

### Changed

- Improve documentation

## [0.1.0] - 2023-01-30

### Added

- Derive macros for `redis` `FromRedisValue` and `ToRedisArgs` traits
- Wrapper type for unwrapping `Json` types
- Add automatic publishing to crates.io
