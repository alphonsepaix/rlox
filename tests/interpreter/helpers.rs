use assert_cmd::assert::Assert;
use assert_cmd::Command;
use std::time::Duration;

pub fn assert_success(source: &str) -> Assert {
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
}

pub fn assert_success_and_check_stdout(source: &str, output: &str) {
    assert_success(source).stdout(predicates::str::contains(output.trim()));
}

pub fn assert_success_and_check_stderr(source: &str, output: &str) {
    assert_success(source).stderr(predicates::str::contains(output.trim()));
}

pub fn assert_failure(source: &str) -> Assert {
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .failure()
}
