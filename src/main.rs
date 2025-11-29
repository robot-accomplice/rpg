use clap::Parser;
use rpg_util::{
    GenerationParams, PasswordArgs, build_char_set, calculate_entropy, column_count,
    generate_passwords, parse_exclude_chars, parse_pattern, print_columns, validate_args,
};

const ASCII_ART: &str = r#"

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
                                     
"#;

const APP_NAME: &str = "RPG";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const BANNER_WIDTH: usize = 79; // Width of the ASCII art banner

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
    after_help = "\n\x1b[1mEXAMPLES:\x1b[0m\n\n  \x1b[36mBasic Usage:\x1b[0m\n    rpg 5                               # Generate 5 passwords\n    rpg 10 --length 20                  # Generate 10 passwords of length 20\n    rpg 25 --table                      # Generate 25 passwords in table format\n\n  \x1b[36mCharacter Customization:\x1b[0m\n    rpg 5 --capitals-off                # Generate without capital letters\n    rpg 5 --numerals-off --symbols-off  # Only alphabetic characters\n    rpg 5 --exclude-chars a-z,0-9       # Exclude ranges of characters\n    rpg 5 --exclude-chars a,b,c         # Exclude specific characters\n    rpg 5 --include-chars a-z,0-9       # Use only specified characters\n\n  \x1b[36mAdvanced Features:\x1b[0m\n    rpg 5 --pattern \"LLLNNNSSS\"         # Pattern-based generation\n    rpg 5 --min-capitals 2 --min-numerals 3  # Minimum requirements\n    rpg 5 --seed 12345                  # Reproducible passwords\n    rpg 1 --copy                        # Copy to clipboard\n    rpg 3 --format json                 # JSON output\n\nFor more information, visit: \x1b[4mhttps://github.com/robot-accomplice/rpg\x1b[0m"
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

    // Print ASCII art banner (unless in quiet mode or JSON format)
    if !args.quiet && args.format != "json" {
        let banner = ASCII_ART.strip_prefix('\n').unwrap_or(ASCII_ART);
        println!(
            "\n{}\n{:>width$}\n",
            banner,
            format!("{} v{}", APP_NAME, APP_VERSION),
            width = BANNER_WIDTH
        );
    }

    // Parse and expand exclude character ranges
    let exclude_chars = match parse_exclude_chars(args.exclude_chars) {
        Ok(chars) => chars,
        Err(e) => {
            eprintln!("Error parsing exclude characters: {}", e);
            std::process::exit(1);
        }
    };

    // Parse and expand include character ranges (if specified)
    let include_chars = if args.include_chars.is_empty() {
        None
    } else {
        match parse_exclude_chars(args.include_chars) {
            Ok(chars) => Some(chars),
            Err(e) => {
                eprintln!("Error parsing include characters: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Parse pattern if specified
    let pattern = if let Some(ref pat_str) = args.pattern {
        match parse_pattern(pat_str) {
            Ok(pat) => Some(pat),
            Err(e) => {
                eprintln!("Error parsing pattern: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    // If pattern is specified, use its length; otherwise use args.length
    let effective_length = pattern
        .as_ref()
        .map(|p| p.len() as u32)
        .unwrap_or(args.length);

    // Convert CLI args to library args
    let password_args = PasswordArgs {
        capitals_off: args.capitals_off,
        numerals_off: args.numerals_off,
        symbols_off: args.symbols_off,
        exclude_chars,
        include_chars,
        min_capitals: args.min_capitals,
        min_numerals: args.min_numerals,
        min_symbols: args.min_symbols,
        pattern: pattern.clone(),
        length: effective_length,
        password_count: args.password_count,
    };

    // Validate arguments
    if let Err(e) = validate_args(&password_args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    // Build character set once (more efficient than building per character)
    let char_set = match build_char_set(&password_args) {
        Ok(set) => set,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Create generation parameters
    let gen_params = GenerationParams {
        length: effective_length,
        count: args.password_count,
        min_capitals: args.min_capitals,
        min_numerals: args.min_numerals,
        min_symbols: args.min_symbols,
        pattern: pattern.clone(),
    };

    // Generate passwords with optional seed
    let passwords = if let Some(seed) = args.seed {
        use rand::{SeedableRng, rngs::StdRng};
        let mut rng = StdRng::seed_from_u64(seed);
        generate_passwords(&char_set, &gen_params, &mut rng)
    } else {
        let mut rng = rand::rng();
        generate_passwords(&char_set, &gen_params, &mut rng)
    };

    // Handle copy to clipboard
    if args.copy && !passwords.is_empty() {
        use clipboard::{ClipboardContext, ClipboardProvider};
        match ClipboardContext::new() {
            Ok(mut ctx) => {
                if ctx.set_contents(passwords[0].clone()).is_ok() && !args.quiet {
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

    // Output passwords in requested format
    match args.format.as_str() {
        "json" => {
            use serde_json::json;
            let json_output = json!({
                "passwords": passwords,
                "count": passwords.len(),
                "length": args.length,
                "entropy_bits": calculate_entropy(char_set.len(), args.length)
            });
            println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        }
        _ => {
            let show_header = !args.quiet;
            if args.table {
                print_columns(passwords, column_count(args.password_count), show_header);
            } else {
                print_columns(passwords, 1, false);
            }
        }
    }
}
