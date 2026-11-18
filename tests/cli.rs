//! Integration tests for the c-hjkl binary: run the actual executable and
//! assert on its exit status and output.

use assert_cmd::Command;
use predicates::prelude::*;

fn c_hjkl() -> assert_cmd::Command {
    Command::cargo_bin("c-hjkl").unwrap()
}

#[test]
fn help_lists_usage_and_the_verbose_flag() {
    c_hjkl()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--verbose"));
}

#[test]
fn help_lists_the_version_flag() {
    c_hjkl()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn version_flag_prints_the_package_name_and_version() {
    c_hjkl()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("c-hjkl"))
        .stdout(predicate::str::contains("0.1"));
}

#[test]
fn unknown_flag_exits_with_an_error() {
    c_hjkl()
        .arg("--bogus")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}
