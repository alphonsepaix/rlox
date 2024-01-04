use assert_cmd::assert::Assert;
use assert_cmd::Command;
use rlox::errors::LoxError;
use rlox::errors::{ScanError, ScanErrorType};
use rlox::scanner::Scanner;
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

pub fn assert_failure_and_check_stderr(source: &str, output: &str) {
    assert_failure(source).stderr(predicates::str::contains(output.trim()));
}

pub fn check_scanner_error(source: &str, expected_type: ScanErrorType) {
    let mut scanner = Scanner::new(source);
    let err = scanner.scan_tokens().err().unwrap();
    if let LoxError::Scan(ScanError { r#type, .. }) = err {
        assert_eq!(r#type, expected_type);
    } else {
        panic!("scanner did not fail for the expected reason");
    }
}
