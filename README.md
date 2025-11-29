
# RPG - Rust Password Generator

```

                            :                  .:
                           .%                  -#
           ..:...          =%*+*-          .*#*#*::           .     .     -++.
 -==+++*****###***+=++==++*%@@@@#+++++++++##%%%%***++++++++++=++====+=++++++%@.
 :-=++*##%%@@@@@%%##**+==+++==+==#%@%%%%#+**++**#######*****+========+==+*%%@#
           .:::::.               -::%###:      .*#-                       .::
                                  ..*###.      :#%-
                                    .#%*       .=+.
                                    :#%#
                                     :::
                                     
```
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-1.0.2-blue.svg)](https://github.com/robot-accomplice/rpg)
[![Crates.io](https://img.shields.io/crates/v/rpg-util.svg)](https://crates.io/crates/rpg-util)
[![Docs.rs](https://docs.rs/rpg-util/badge.svg)](https://docs.rs/rpg-util)
[![codecov](https://codecov.io/gh/robot-accomplice/rpg/branch/main/graph/badge.svg)](https://codecov.io/gh/robot-accomplice/rpg)


A fast, secure, and customizable command-line password generator written in Rust.

## Features

- **ASCII art banner**: Displays a stylized RPG launcher banner on execution
- **Customizable character sets**: Control which types of characters to include (lowercase, uppercase, numerals, symbols)
- **Character exclusion**: Exclude specific characters or ranges from generated passwords
- **Character inclusion**: Restrict passwords to only specific character sets
- **Pattern-based generation**: Generate passwords from patterns (e.g., `LLLNNNSSS` for 3 lowercase, 3 numeric, 3 symbols)
- **Minimum requirements**: Enforce minimum counts of capitals, numerals, or symbols
- **Reproducible passwords**: Use seeds to generate the same passwords repeatedly
- **Multiple output formats**: Text (default) or JSON with entropy information
- **Clipboard integration**: Copy passwords directly to clipboard
- **Table output**: Display multiple passwords in a formatted table
- **Quiet mode**: Suppress banner and headers for script-friendly output
- **Uniform distribution**: Passwords are generated with uniform character distribution
- **Fast and efficient**: Pre-allocated memory and optimized character set building
- **Well-tested**: Comprehensive unit tests (17+) and integration tests

## Installation

### From Crates.io

```bash
cargo install rpg-util
```

### From Source

```bash
git clone https://github.com/robot-accomplice/rpg.git
cd rpg
cargo build --release
```

The binary will be available at `target/release/rpg`.

## Usage

### Basic Usage

Generate 5 passwords with default settings (16 characters, all character types):

```bash
rpg 5
```

### Options

- `-l, --length <LENGTH>`: Set password length (default: 16, max: 10,000)
- `-c, --capitals-off`: Disable capital letters
- `-n, --numerals-off`: Disable numerals
- `-s, --symbols-off`: Disable symbols
- `-e, --exclude-chars <CHARS>`: Exclude specific characters or ranges (e.g., `a-z`, `0-9`)
- `--include-chars <CHARS>`: Include only specific characters or ranges (overrides type flags)
- `--min-capitals <N>`: Minimum number of capital letters required
- `--min-numerals <N>`: Minimum number of numerals required
- `--min-symbols <N>`: Minimum number of symbols required
- `-t, --table`: Display passwords in table format
- `-q, --quiet`: Suppress banner and header output
- `--seed <SEED>`: Seed for reproducible password generation
- `--format <FORMAT>`: Output format: "text" (default) or "json"
- `--copy`: Copy first password to clipboard
- `--pattern <PATTERN>`: Generate passwords from a pattern (L=lowercase, U=uppercase, N=numeric, S=symbol)

### Examples

Generate 10 passwords of length 20:

```bash
rpg 10 --length 20
```

Generate 5 passwords without capital letters:

```bash
rpg 5 --capitals-off
```

Generate 5 passwords with only alphabetic characters:

```bash
rpg 5 --numerals-off --symbols-off
```

Generate 5 passwords excluding specific characters or ranges:

```bash
rpg 5 --exclude-chars a-z          # Exclude all lowercase letters
rpg 5 --exclude-chars 0-9          # Exclude all digits
rpg 5 --exclude-chars a-z,0-9      # Exclude ranges and individual chars
rpg 5 --exclude-chars a,b,c        # Exclude individual characters
rpg 5 --exclude-chars a-c,x,0-2    # Mix of ranges and individual chars
```

Display passwords in table format:

```bash
rpg 10 --table
```

Generate passwords with custom length and character restrictions:

```bash
rpg 8 --length 24 --capitals-off --exclude-chars 0,O,1,l
```

Generate passwords with minimum requirements:

```bash
rpg 5 --length 20 --min-capitals 2 --min-numerals 3 --min-symbols 1
```

Generate reproducible passwords with seed:

```bash
rpg 5 --seed 12345
```

Output in JSON format:

```bash
rpg 3 --format json
```

Copy first password to clipboard:

```bash
rpg 1 --copy
```

Use only specific characters:

```bash
rpg 5 --include-chars a-z,0-9
```

Generate passwords from a pattern:

```bash
rpg 5 --pattern "LLLNNNSSS"  # 3 lowercase, 3 numeric, 3 symbols
rpg 5 --pattern "UUUlllnnn"  # 3 uppercase, 3 lowercase, 3 numeric
```

## Character Sets

The generator uses the following character ranges:

- **Lowercase letters**: `a-z` (always included)
- **Uppercase letters**: `A-Z` (can be disabled with `--capitals-off`)
- **Numerals**: `0-9` (can be disabled with `--numerals-off`)
- **Symbols**: All ASCII printable symbols (can be disabled with `--symbols-off`)
  - `!"#$%&'()*+,-./` (33-47)
  - `:;<=>?@` (58-64)
  - `[\]^_\`` (91-96)
  - `{|}~` (123-126)

## Error Handling

The generator validates inputs and provides clear error messages:

- Invalid password length or count
- All character types disabled
- All characters excluded

## Performance

- Pre-allocated memory for efficient generation
- O(1) character exclusion checking using HashSet
- Single character set build for all passwords
- Optimized random sampling
- Benchmarked with criterion for performance tracking

## Library Usage

RPG can also be used as a library in your Rust projects:

```toml
[dependencies]
rpg-util = "1.0.2"
```

```rust
use rpg_util::{GenerationParams, PasswordArgs, build_char_set, generate_passwords, parse_pattern};
use rand::Rng;

let args = PasswordArgs {
    capitals_off: false,
    numerals_off: false,
    symbols_off: false,
    exclude_chars: vec![],
    include_chars: None,
    min_capitals: None,
    min_numerals: None,
    min_symbols: None,
    pattern: None,
    length: 16,
    password_count: 1,
};

let char_set = build_char_set(&args)?;
let mut rng = rand::rng();

let gen_params = GenerationParams {
    length: 16,
    count: 1,
    min_capitals: None,
    min_numerals: None,
    min_symbols: None,
    pattern: None,
};
let passwords = generate_passwords(&char_set, &gen_params, &mut rng);

// Or use pattern-based generation
let pattern = parse_pattern("LLLNNNSSS")?;
let gen_params = GenerationParams {
    length: 9,
    count: 1,
    min_capitals: None,
    min_numerals: None,
    min_symbols: None,
    pattern: Some(pattern),
};
let passwords = generate_passwords(&char_set, &gen_params, &mut rng);
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test

# Run with output
cargo test -- --nocapture
```

## Benchmarks

Run performance benchmarks:

```bash
cargo bench
```

This will generate HTML reports in `target/criterion/`.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

1. Clone the repository
2. Run tests: `cargo test`
3. Run clippy: `cargo clippy`
4. Format code: `cargo fmt`

### Pre-commit Hooks

Optional pre-commit hooks are available to ensure code quality. See [PRE_COMMIT_HOOKS.md](PRE_COMMIT_HOOKS.md) for detailed documentation.

Quick setup:

```bash
pip install pre-commit
pre-commit install
```

This will run formatting, linting, and tests before each commit.
