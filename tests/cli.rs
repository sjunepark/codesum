use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_should_fail_when_no_arguments_are_provided() {
    let mut cmd = Command::cargo_bin("codesum").unwrap();
    let assert = cmd.assert();
    assert.failure();
}

#[test]
fn test_should_properly_print_help() {
    let mut cmd = Command::cargo_bin("codesum").unwrap();
    let assert = cmd.arg("--help").assert();

    assert
        .success()
        .stdout(predicates::str::contains("codesum").and(predicates::str::contains("help")));
}
