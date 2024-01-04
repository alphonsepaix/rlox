use crate::helpers::{assert_failure_and_check_stderr, check_scanner_error};
use claim::assert_ok;
use rlox::errors::ScanErrorType::*;
use rlox::scanner::*;

#[test]
fn simple_expression_tokenized_correctly() {
    let input = r#"let name = "Alphonse";"#;
    let mut scanner = Scanner::new(input);

    assert_ok!(scanner.scan_tokens());

    let types = vec![
        TokenType::Let,
        TokenType::Identifier("name".to_string()),
        TokenType::Equal,
        TokenType::Str("Alphonse".to_string()),
        TokenType::Semicolon,
        TokenType::Eof,
    ];

    for (_expected_type, token_type) in types
        .iter()
        .zip(scanner.tokens.into_iter().map(|t| t.r#type))
    {
        assert_eq!(_expected_type, &token_type);
    }
}

#[test]
fn unterminated_string_returns_error() {
    let source = r#"let name = "Alphonse;"#;
    check_scanner_error(source, UnterminatedString);
    let source = r#"
let x = "Louis";
let a = 4;
print("Alphonse + Louis);
"#;
    assert_failure_and_check_stderr(source, "unterminated string");
}

#[test]
fn invalid_number_returns_error() {
    let source = "let x = 1253.f";
    check_scanner_error(source, InvalidNumber);
}

#[test]
fn invalid_expressions_return_error() {
    let cases = [
        ("let x = 1253.f", InvalidNumber),
        (r#"let x = 12. name = "Alphonse""#, InvalidNumber),
        ("let @pi = 3.415", UnexpectedCharacter),
        (
            r#"let name = "Alphonse; var x = 3.1415"#,
            UnterminatedString,
        ),
    ];
    for (source, error_type) in cases {
        check_scanner_error(source, error_type);
    }
}
