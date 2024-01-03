use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;

#[test]
fn if_statement() {
    let source = r#"
var x = 3;
if (x > 9) {
    print "x > 9!";
} else {
    print "x <= 9!";
}
"#;
    let output = "x <= 9!
";
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c").arg(source).assert().success().stdout(output);
}

#[test]
fn for_statement() {
    let source = r#"
var i = 0;
for (; i < 10; i = i + 1)
    if (i > 5)
        print i;
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    let output = "6
7
8
9
";
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
        .stdout(output);
}

#[test]
fn for_constant_expr() {
    let source = r#"
for (var i = 0; i < 10; i = i + 1)
    print "Alphonse";
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    let output = "Alphonse\n".repeat(10);
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
        .stdout(output);
}

#[test]
fn while_statement() {
    let source = r#"
var i = 3;
while (i < 8) {
    i = i + 1;
    if (i > 4)
        print i;
}
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    let output = "5
6
7
8
";
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
        .stdout(output);
}

#[test]
fn continue_statement() {
    let source = r#"
for (var i = 0; i < 20; i = i + 1) {
    if (i <= 10)
        continue;
    print i;
}
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    let output = "11
12
13
14
15
16
17
18
19
";
    cmd.arg("-c").arg(source).assert().success().stdout(output);
}

#[test]
fn break_statement_in_while_loop() {
    let source = r#"
var i = 0;
while (i < 10) {
    i = i + 2;
    if (i >= 7)
        break;
}
print i;
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c").arg(source).assert().success().stdout("8\n");
}

#[test]
fn break_outside_loop_is_detected_correctly() {
    let source = r#"
for (var i = 0; i < 10; i = i + 1) {
    // do complicated stuff
}
break;
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
        .stderr(predicate::str::contains("`break` outside loop"));
}

#[test]
fn continue_outside_loop_is_detected_correctly() {
    let source = r#"
for (var i = 0; i < 10; i = i + 1) {
    // do complicated stuff
    break;
}
continue;
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
        .stderr(predicate::str::contains("`continue` outside loop"));
}

#[test]
fn inner_loops() {
    let source = r#"
for (;;) {
    for (;;) {
        for (;;) {
            for (;;) {
                break;
            }
            break;
        }
        break;
    }
    break;
}
print "Outside!";
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
        .stdout("Outside!\n");
}

#[test]
fn infinite_loops() {
    let source = r#"
for (;;)         // infinite loop
print "Hello!";
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .failure();

    let source = r#"
var i = 0;
while (i < 10)
    i = i - 1;      // user meant to increment i surely
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .failure();
}

#[test]
fn fibonacci() {
    let source = r#"
var a = 0;
var b = 1;
var n = 10;
for (var i = 0; i < n - 1; i = i + 1) {
    var tmp = b;
    b = b + a;
    a = tmp;
}
print a;    // the n-th Fibonacci number
"#;
    let mut cmd = Command::cargo_bin("rlox").unwrap();
    cmd.arg("-c")
        .arg(source)
        .timeout(Duration::from_secs(1))
        .assert()
        .success()
        .stdout("34\n");
}
