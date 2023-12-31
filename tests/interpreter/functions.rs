use crate::helpers::{assert_success_and_check_stderr, assert_success_and_check_stdout};

#[test]
fn fibonacci() {
    let source = r#"
fn fibonacci(n) {
    let a = 0;
    let b = 1;
    for (let i = 0; i < n - 1; i = i + 1) {
        let tmp = b;
        b = b + a;
        a = tmp;
    }
    print(a);
}
for (let n = 1; n <= 10; n = n + 1)
    fibonacci(n);
"#;
    let output = r#"
0
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
    assert_success_and_check_stdout(source, output);
}

#[test]
fn fibonacci_recursion() {
    let source = r#"
fn fibonacci(n) {
    if (n == 1) return 0;
    if (n == 2) return 1;
    return fibonacci(n - 1) + fibonacci(n - 2);
}
print(fibonacci(8));
"#;
    assert_success_and_check_stdout(source, "13");
}

#[test]
fn return_outside_of_function_detected_correctly() {
    let source = r#"
fn foo() {
    print("Hi!");
}
return;
"#;
    assert_success_and_check_stderr(source, "`return` outside function");
}
