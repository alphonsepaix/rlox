use crate::helpers::{
    assert_failure, assert_success_and_check_stderr, assert_success_and_check_stdout,
};

#[test]
fn if_statement() {
    let source = r#"
let x = 3;
if (x > 9) {
    print("x > 9!");
} else {
    print("x <= 9!");
}
"#;
    let output = "x <= 9!
";
    assert_success_and_check_stdout(source, output);
}

#[test]
fn for_statement() {
    let source = r#"
let i = 0;
for (; i < 10; i = i + 1)
    if (i > 5)
        print(i);
"#;
    let output = "
6
7
8
9
";
    assert_success_and_check_stdout(source, output);
}

#[test]
fn for_constant_expr() {
    let source = r#"
for (let i = 0; i < 10; i = i + 1)
    print("Alphonse");
"#;
    let output = "Alphonse\n".repeat(10);
    assert_success_and_check_stdout(source, &output);
}

#[test]
fn while_statement() {
    let source = r#"
let i = 3;
while (i < 8) {
    i = i + 1;
    if (i > 4)
        print(i);
}
"#;
    let output = "
5
6
7
8
";
    assert_success_and_check_stdout(source, &output);
}

#[test]
fn continue_statement() {
    let source = r#"
for (let i = 0; i < 20; i = i + 1) {
    if (i <= 10)
        continue;
    print(i);
}
"#;
    let output = "
11
12
13
14
15
16
17
18
19
";
    assert_success_and_check_stdout(source, &output);
}

#[test]
fn break_statement_in_while_loop() {
    let source = r#"
let i = 0;
while (i < 10) {
    i = i + 2;
    if (i >= 7)
        break;
}
print(i);
"#;
    assert_success_and_check_stdout(source, "8");
}

#[test]
fn break_outside_loop_is_detected_correctly() {
    let source = r#"
for (let i = 0; i < 10; i = i + 1) {
    // do complicated stuff
}
break;
"#;
    assert_success_and_check_stderr(source, "`break` outside loop");
}

#[test]
fn continue_outside_loop_is_detected_correctly() {
    let source = r#"
for (let i = 0; i < 10; i = i + 1) {
    // do complicated stuff
    break;
}
continue;
"#;
    assert_success_and_check_stderr(source, "`continue` outside loop");
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
print("Outside!");
"#;
    assert_success_and_check_stdout(source, "Outside!");
}

#[test]
fn infinite_loops() {
    let source = r#"
for (;;)         // infinite loop
    ;
print("Outside!");
"#;
    assert_failure(source);

    let source = r#"
let i = 0;
while (i < 10)
    i = i - 1;      // user meant to increment i surely
"#;
    assert_failure(source);
}

#[test]
fn fibonacci() {
    let source = r#"
let a = 0;
let b = 1;
let n = 10;
for (let i = 0; i < n - 1; i = i + 1) {
    let tmp = b;
    b = b + a;
    a = tmp;
}
print(a);    // the n-th Fibonacci number
"#;
    assert_success_and_check_stdout(source, "34");
}
