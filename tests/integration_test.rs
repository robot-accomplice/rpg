use std::process::Command;

#[test]
fn test_basic_generation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["3", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(
        lines.len(),
        3,
        "Expected 3 passwords, got {}: {:?}",
        lines.len(),
        stdout
    );
}

#[test]
fn test_length_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--length", "20", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let passwords: Vec<&str> = stdout
        .lines()
        .filter(|l| {
            !l.is_empty()
                && !l.contains("Printing")
                && !l.contains("RPG v")
                && !(l.chars().filter(|&c| c == '@').count() > 5) // Filter banner lines (many @ chars)
        })
        .collect();

    assert!(
        !passwords.is_empty(),
        "No passwords found in output: {:?}",
        stdout
    );
    let password = passwords[0].trim();
    assert_eq!(
        password.len(),
        20,
        "Expected password length 20, got {}: {:?}",
        password.len(),
        password
    );
}

#[test]
fn test_seed_reproducibility() {
    let output1 = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--seed", "12345", "--quiet"])
        .output()
        .expect("Failed to execute command");

    let output2 = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--seed", "12345", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    let pass1: Vec<&str> = stdout1
        .lines()
        .filter(|l| {
            !l.is_empty()
                && !l.contains("Printing")
                && !l.contains("RPG v")
                && !(l.chars().filter(|&c| c == '@').count() > 5) // Filter banner lines (many @ chars)
        })
        .collect();
    let pass2: Vec<&str> = stdout2
        .lines()
        .filter(|l| {
            !l.is_empty()
                && !l.contains("Printing")
                && !l.contains("RPG v")
                && !(l.chars().filter(|&c| c == '@').count() > 5) // Filter banner lines (many @ chars)
        })
        .collect();

    assert_eq!(pass1, pass2);
}

#[test]
fn test_cli_invalid_exclude_chars() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--exclude-chars", "z-a", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail with invalid range");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Invalid range") || stderr.contains("Error parsing exclude characters")
    );
}

#[test]
fn test_cli_invalid_include_chars() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--include-chars", "z-a", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail with invalid range");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Invalid range") || stderr.contains("Error parsing include characters")
    );
}

#[test]
fn test_cli_invalid_pattern() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--pattern", "LLX", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail with invalid pattern");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Invalid pattern character") || stderr.contains("Error parsing pattern")
    );
}

#[test]
fn test_cli_invalid_length_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--length", "0", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail with length 0");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Password length must be greater than 0"));
}

#[test]
fn test_cli_invalid_length_too_long() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--length", "10001", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail with length > 10000");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("exceeds maximum of 10,000"));
}

#[test]
fn test_cli_json_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["2", "--length", "10", "--format", "json", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should be valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");
    assert!(json.get("passwords").is_some());
    assert!(json.get("count").is_some());
    assert!(json.get("length").is_some());
    assert!(json.get("entropy_bits").is_some());

    let passwords = json.get("passwords").unwrap().as_array().unwrap();
    assert_eq!(passwords.len(), 2);
    assert_eq!(passwords[0].as_str().unwrap().len(), 10);
}

#[test]
fn test_cli_table_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["6", "--table", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

    // Should have passwords in table format (may be on multiple lines in columns)
    // With 6 passwords and 2 columns, we'd expect at least 3 lines
    assert!(
        lines.len() >= 3,
        "Expected at least 3 lines in table format, got {}",
        lines.len()
    );
    // Verify passwords are present by checking non-empty content
    assert!(!stdout.trim().is_empty(), "Output should contain passwords");
}

#[test]
fn test_cli_table_with_header() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["6", "--table"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should contain header when not in quiet mode
    assert!(stdout.contains("Printing"));
}

#[test]
fn test_cli_quiet_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["3", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should not contain banner or header in quiet mode
    assert!(!stdout.contains("RPG v"));
    assert!(!stdout.contains("Printing"));

    // Should only have passwords
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_cli_pattern_generation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--pattern", "LLLNNNSSS", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let password = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .next()
        .unwrap()
        .trim();

    assert_eq!(password.len(), 9);
    // Verify pattern was followed (can't predict exact chars but can verify types)
    let chars: Vec<char> = password.chars().collect();
    assert!(chars[0].is_ascii_lowercase());
    assert!(chars[1].is_ascii_lowercase());
    assert!(chars[2].is_ascii_lowercase());
    assert!(chars[3].is_ascii_digit());
    assert!(chars[4].is_ascii_digit());
    assert!(chars[5].is_ascii_digit());
    assert!(!chars[6].is_alphanumeric());
    assert!(!chars[7].is_alphanumeric());
    assert!(!chars[8].is_alphanumeric());
}

#[test]
fn test_cli_pattern_case_insensitive() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--pattern", "lllununss", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let password = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .next()
        .unwrap()
        .trim();

    assert_eq!(password.len(), 9);
}

#[test]
fn test_cli_minimum_requirements() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&[
            "1",
            "--length",
            "10",
            "--min-capitals",
            "2",
            "--min-numerals",
            "2",
            "--min-symbols",
            "2",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let password = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .next()
        .unwrap()
        .trim();

    assert_eq!(password.len(), 10);
    let capitals = password.chars().filter(|c| c.is_ascii_uppercase()).count();
    let numerals = password.chars().filter(|c| c.is_ascii_digit()).count();
    let symbols = password.chars().filter(|c| !c.is_alphanumeric()).count();

    assert!(capitals >= 2);
    assert!(numerals >= 2);
    assert!(symbols >= 2);
}

#[test]
fn test_cli_seed_reproducibility_with_options() {
    let output1 = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&[
            "1",
            "--seed",
            "999",
            "--length",
            "20",
            "--min-capitals",
            "3",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute command");

    let output2 = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&[
            "1",
            "--seed",
            "999",
            "--length",
            "20",
            "--min-capitals",
            "3",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();
    let pass1: Vec<&str> = stdout1.lines().filter(|l| !l.is_empty()).collect();
    let pass2: Vec<&str> = stdout2.lines().filter(|l| !l.is_empty()).collect();

    assert_eq!(pass1, pass2);
}

#[test]
fn test_cli_include_chars() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&[
            "1",
            "--include-chars",
            "a,b,c,1,2,3",
            "--length",
            "10",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let password = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .next()
        .unwrap()
        .trim();

    assert_eq!(password.len(), 10);
    // All characters should be from the include set
    for c in password.chars() {
        assert!(matches!(c, 'a' | 'b' | 'c' | '1' | '2' | '3'));
    }
}

#[test]
fn test_cli_include_chars_with_exclude() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&[
            "1",
            "--include-chars",
            "a,b,c",
            "--exclude-chars",
            "a",
            "--length",
            "10",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let password = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .next()
        .unwrap()
        .trim();

    assert_eq!(password.len(), 10);
    // Should only contain 'b' or 'c'
    for c in password.chars() {
        assert!(matches!(c, 'b' | 'c'));
    }
}

#[test]
fn test_cli_include_chars_with_range() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&[
            "1",
            "--include-chars",
            "a-z,0-9",
            "--length",
            "10",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let password = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .next()
        .unwrap()
        .trim();

    assert_eq!(password.len(), 10);
    // All characters should be lowercase or digit
    for c in password.chars() {
        assert!(c.is_ascii_lowercase() || c.is_ascii_digit());
    }
}

#[test]
fn test_cli_character_type_combinations() {
    // Test with capitals off
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--capitals-off", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let password = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .next()
        .unwrap()
        .trim();

    // Should not contain uppercase letters
    assert!(!password.chars().any(|c| c.is_ascii_uppercase()));
}

#[test]
fn test_cli_pattern_overrides_length() {
    // Pattern length should override --length option
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--pattern", "LLL", "--length", "20", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let password = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .next()
        .unwrap()
        .trim();

    // Should be length 3 (pattern length), not 20
    assert_eq!(password.len(), 3);
}

#[test]
fn test_cli_exclude_chars_with_range() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpg"))
        .args(&["1", "--exclude-chars", "a-z", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let password = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .next()
        .unwrap()
        .trim();

    // Should not contain lowercase letters
    assert!(!password.chars().any(|c| c.is_ascii_lowercase()));
}
