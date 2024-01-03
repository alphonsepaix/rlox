use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;

#[test]
fn fibonacci() {
    let source = r#"
fun fibonacci(n) {
    var a = 0;
    var b = 1;
    for (var i = 0; i < n - 1; i = i + 1) {
        var tmp = b;
        b = b + a;
        a = tmp;
    }
    print a;
}
for (var n = 1; n <= 10; n = n + 1)
    fibonacci(n);
"#;
    let output = r#"0
1
1
2
3
5
8
13
21
34
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c").arg(source).assert().success().stdout(output);
}

#[test]
fn fibonacci_recursion() {
    let source = r#"
fun fibonacci(n) {
    if (n == 1) return 0;
    if (n == 2) return 1;
    return fibonacci(n - 1) + fibonacci(n - 2);
}
print fibonacci(8);
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
        .stdout("13\n");
}

#[test]
fn return_outside_of_function_detected_correctly() {
    let source = r#"
fun foo() {
    print "Hi!";
}
return;
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
        .stderr(predicate::str::contains("`return` outside function"));
}
