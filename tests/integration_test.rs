use std::process::Command;

#[test]
fn test_basic_generation() {
    let output = Command::new("./target/release/rpg")
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
    let output = Command::new("./target/release/rpg")
        .args(&["1", "--length", "20", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let passwords: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.is_empty() && !l.contains("Printing") && !l.contains("@"))
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
    let output1 = Command::new("./target/release/rpg")
        .args(&["1", "--seed", "12345", "--quiet"])
        .output()
        .expect("Failed to execute command");

    let output2 = Command::new("./target/release/rpg")
        .args(&["1", "--seed", "12345", "--quiet"])
        .output()
        .expect("Failed to execute command");

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    let pass1: Vec<&str> = stdout1
        .lines()
        .filter(|l| !l.is_empty() && !l.contains("Printing") && !l.contains("@"))
        .collect();
    let pass2: Vec<&str> = stdout2
        .lines()
        .filter(|l| !l.is_empty() && !l.contains("Printing") && !l.contains("@"))
        .collect();

    assert_eq!(pass1, pass2);
}
