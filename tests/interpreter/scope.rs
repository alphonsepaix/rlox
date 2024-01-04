use crate::helpers::assert_success_and_check_stdout;

#[test]
fn nested_blocks() {
    let source = r#"
let a = "global a";
let b = "global b";
let c = "global c";
{
  let a = "outer a";
  let b = "outer b";
  {
    let a = "inner a";
    print(a);
    print(b);
    print(c);
  }
  print(a);
  print(b);
  print(c);
}
print(a);
print(b);
print(c);
"#;
    let output = "
inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
";
    assert_success_and_check_stdout(source, output);
}
