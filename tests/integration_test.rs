use std::process::Command;

#[test]
fn test_basic_generation() {
    let output = Command::new("cargo")
        .args(&["run", "--release", "--", "3"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_length_option() {
    let output = Command::new("cargo")
        .args(&["run", "--release", "--", "1", "--length", "20"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let line = stdout
        .lines()
        .find(|l| !l.is_empty() && !l.contains("Printing"))
        .unwrap();
    assert_eq!(line.len(), 20);
}

#[test]
fn test_seed_reproducibility() {
    let output1 = Command::new("cargo")
        .args(&["run", "--release", "--", "1", "--seed", "12345"])
        .output()
        .expect("Failed to execute command");

    let output2 = Command::new("cargo")
        .args(&["run", "--release", "--", "1", "--seed", "12345"])
        .output()
        .expect("Failed to execute command");

    assert!(output1.status.success());
    assert!(output2.status.success());

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    let pass1: Vec<&str> = stdout1
        .lines()
        .filter(|l| !l.is_empty() && !l.contains("Printing"))
        .collect();
    let pass2: Vec<&str> = stdout2
        .lines()
        .filter(|l| !l.is_empty() && !l.contains("Printing"))
        .collect();

    assert_eq!(pass1, pass2);
}
