# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-11-28

### Added
- Initial release of RPG (Rust Password Generator)
- Customizable character sets (lowercase, uppercase, numerals, symbols)
- Character exclusion with range support (e.g., `a-z`, `0-9`)
- Character inclusion option (`--include-chars`)
- Minimum character type requirements (`--min-capitals`, `--min-numerals`, `--min-symbols`)
- Table output format (`--table`)
- Quiet mode (`--quiet`)
- Seed support for reproducible passwords (`--seed`)
- JSON output format (`--format json`)
- Copy to clipboard functionality (`--copy`)
- Password entropy calculation
- Comprehensive unit tests
- Integration tests
- Performance benchmarks
- Maximum password length validation (10,000 characters)
- Improved error messages with helpful hints
- MIT License
- Complete documentation

### Features
- Fast and efficient password generation
- Uniform character distribution
- O(1) character exclusion checking using HashSet
- Pre-allocated memory for optimal performance
- Well-tested with 17+ unit tests

