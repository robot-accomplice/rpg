# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2025-01-29

### Changed
- Updated ASCII art banner design
- Improved help output formatting with better organization and examples
- Enhanced banner display with version caption below ASCII art
- Refactored banner formatting for better consistency
- Updated integration test filters for improved clarity

## [1.0.0] - 2024-11-28

### Added
- Initial release of RPG (Rust Password Generator)
- ASCII art banner displayed on program execution
- Customizable character sets (lowercase, uppercase, numerals, symbols)
- Character exclusion with range support (e.g., `a-z`, `0-9`)
- Character inclusion option (`--include-chars`) to restrict to specific character sets
- Minimum character type requirements (`--min-capitals`, `--min-numerals`, `--min-symbols`)
- Pattern-based password generation (`--pattern`) with L (lowercase), U (uppercase), N (numeric), S (symbol)
- Table output format (`--table`)
- Quiet mode (`--quiet`) to suppress banner and headers
- Seed support for reproducible passwords (`--seed`)
- JSON output format (`--format json`) with entropy information
- Copy to clipboard functionality (`--copy`)
- Password entropy calculation (displayed in JSON output)
- Comprehensive unit tests (17+ tests)
- Integration tests for CLI workflows
- Performance benchmarks using criterion
- Maximum password length validation (10,000 characters)
- Improved error messages with helpful hints and suggestions
- MIT License
- Complete documentation (README, INSTALL, PUBLISHING, PRE_COMMIT_HOOKS guides)
- Man page (rpg.1) for Unix/Linux systems
- GitHub Actions CI/CD pipeline
- Code coverage setup with Codecov

### Features
- Fast and efficient password generation
- Uniform character distribution
- O(1) character exclusion checking using HashSet
- Pre-allocated memory for optimal performance
- Well-tested with 17+ unit tests

