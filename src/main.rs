use clap::Parser;
use rpg_util::{
    CharacterSet, PasswordConfig, PasswordError, calculate_entropy, column_count,
    generate_passwords, parse_char_specs, parse_pattern, print_columns,
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const BANNER_WIDTH: usize = 79;

fn format_banner_with_caption() -> String {
    let banner = include_str!("../banner.txt");
    let caption = format!("RPG v{}", APP_VERSION);
    format!(
        "\n{}\n{:>width$}\n",
        banner.trim_end(),
        caption,
        width = BANNER_WIDTH
    )
}

/// RPG - Rust Password Generator
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Rust Password Generator - A fast and customizable password generator",
    long_about = None,
    before_help = format_banner_with_caption(),
    after_help = "\n\x1b[1mEXAMPLES:\x1b[0m\n\n  \x1b[36mBasic Usage:\x1b[0m\n    rpg 5                               # Generate 5 passwords\n    rpg 10 --length 20                  # Generate 10 passwords of length 20\n    rpg 25 --table                      # Generate 25 passwords in table format\n\n  \x1b[36mCharacter Customization:\x1b[0m\n    rpg 5 --capitals-off                # Generate without capital letters\n    rpg 5 --numerals-off --symbols-off  # Only alphabetic characters\n    rpg 5 --exclude-chars a-z,0-9       # Exclude ranges of characters\n    rpg 5 --exclude-chars a,b,c         # Exclude specific characters\n    rpg 5 --include-chars a-z,0-9       # Use only specified characters\n\n  \x1b[36mAdvanced Features:\x1b[0m\n    rpg 5 --pattern \"LLLNNNSSS\"         # Pattern-based generation\n    rpg 5 --min-capitals 2              # Minimum capital letters\n    rpg 5 --min-numerals 3              # Minimum numerals\n    rpg 5 --seed 12345                  # Reproducible passwords\n    rpg 1 --copy                        # Copy to clipboard\n    rpg 3 --format json                 # JSON output\n\nFor more information, visit: \x1b[4mhttps://github.com/robot-accomplice/rpg\x1b[0m"
)]
struct Args {
    /// Disable capital letters
    #[arg(short, long, default_value = "false")]
    capitals_off: bool,

    /// Disable numerals
    #[arg(short, long, default_value = "false")]
    numerals_off: bool,

    /// Disable symbols
    #[arg(short, long, default_value = "false")]
    symbols_off: bool,

    /// Exclude specific characters or ranges (supports multiple times, comma-separated, and ranges)
    #[arg(short, long, value_delimiter = ',')]
    exclude_chars: Vec<String>,

    /// Include only specific characters or ranges (overrides character type flags)
    #[arg(long, value_delimiter = ',')]
    include_chars: Vec<String>,

    /// Minimum number of capital letters required
    #[arg(long)]
    min_capitals: Option<u32>,

    /// Minimum number of numerals required
    #[arg(long)]
    min_numerals: Option<u32>,

    /// Minimum number of symbols required
    #[arg(long)]
    min_symbols: Option<u32>,

    /// Length of the password
    #[arg(short, long, default_value = "16")]
    length: u32,

    /// Number of passwords to generate
    #[arg(required = true)]
    password_count: u32,

    /// Print passwords in a table format
    #[arg(short, long, default_value = "false")]
    table: bool,

    /// Suppress header output (quiet mode)
    #[arg(short, long, default_value = "false")]
    quiet: bool,

    /// Seed for random number generator (for reproducible passwords)
    #[arg(long)]
    seed: Option<u64>,

    /// Output format: "text" (default) or "json"
    #[arg(long, default_value = "text")]
    format: String,

    /// Copy first password to clipboard
    #[arg(long, default_value = "false")]
    copy: bool,

    /// Pattern for password generation (L=lowercase, U=uppercase, N=numeric, S=symbol)
    /// Example: "LLLNNNSSS" generates 3 lowercase, 3 numeric, 3 symbols
    #[arg(long)]
    pattern: Option<String>,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let config = build_config(&args)?;
    config.validate()?;

    let char_set = CharacterSet::from_config(&config)?;
    let passwords = generate(&char_set, &config, args.seed);

    copy_to_clipboard(&passwords, args.copy, args.quiet);
    output_passwords(&passwords, &char_set, &args);

    Ok(())
}

fn build_config(args: &Args) -> Result<PasswordConfig, PasswordError> {
    let exclude_chars = parse_char_specs(args.exclude_chars.clone())?;

    let include_chars = if args.include_chars.is_empty() {
        None
    } else {
        Some(parse_char_specs(args.include_chars.clone())?)
    };

    let pattern = match args.pattern {
        Some(ref pat_str) => Some(parse_pattern(pat_str)?),
        None => None,
    };

    let mut config = PasswordConfig {
        capitals_off: args.capitals_off,
        numerals_off: args.numerals_off,
        symbols_off: args.symbols_off,
        exclude_chars,
        include_chars,
        min_capitals: args.min_capitals,
        min_numerals: args.min_numerals,
        min_symbols: args.min_symbols,
        pattern,
        length: args.length,
        count: args.password_count,
    };

    // Pattern length overrides --length
    config.length = config.effective_length();

    Ok(config)
}

fn generate(char_set: &CharacterSet, config: &PasswordConfig, seed: Option<u64>) -> Vec<String> {
    if let Some(seed) = seed {
        use rand::{SeedableRng, rngs::StdRng};
        let mut rng = StdRng::seed_from_u64(seed);
        generate_passwords(char_set, config, &mut rng)
    } else {
        let mut rng = rand::rng();
        generate_passwords(char_set, config, &mut rng)
    }
}

fn copy_to_clipboard(passwords: &[String], should_copy: bool, quiet: bool) {
    #[cfg(feature = "clipboard")]
    if should_copy && !passwords.is_empty() {
        use arboard::Clipboard;
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if clipboard.set_text(&passwords[0]).is_ok() && !quiet {
                    eprintln!("Password copied to clipboard");
                }
            }
            Err(_) => {
                eprintln!(
                    "Warning: Could not copy to clipboard (clipboard functionality not available)"
                );
            }
        }
    }
    #[cfg(not(feature = "clipboard"))]
    if should_copy && !quiet {
        eprintln!("Warning: Clipboard functionality not available (clipboard feature not enabled)");
    }
}

fn output_passwords(passwords: &[String], char_set: &CharacterSet, args: &Args) {
    match args.format.as_str() {
        "json" => output_json(passwords, char_set, args.length),
        _ => output_text(passwords, args),
    }
}

fn output_json(passwords: &[String], char_set: &CharacterSet, length: u32) {
    use serde_json::json;
    let json_output = json!({
        "passwords": passwords,
        "count": passwords.len(),
        "length": length,
        "entropy_bits": calculate_entropy(char_set.len(), length)
    });
    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
}

fn output_text(passwords: &[String], args: &Args) {
    let show_header = !args.quiet;
    if args.table {
        print_columns(
            passwords.to_vec(),
            column_count(args.password_count),
            show_header,
        );
    } else {
        print_columns(passwords.to_vec(), 1, false);
    }
}
