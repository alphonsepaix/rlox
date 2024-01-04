use claim::assert_ok;
use rlox::errors::LoxError::*;
use rlox::errors::ScanErrorType::*;
use rlox::errors::*;
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
        TokenType::String("Alphonse".to_string()),
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
    let input = r#"let name = "Alphonse;"#;
    let mut scanner = Scanner::new(input);
    let err = scanner.scan_tokens().err().unwrap();
    assert!(matches!(
        err,
        Scan(ScanError {
            r#type: UnterminatedString,
            ..
        })
    ));
}

#[test]
fn invalid_number_returns_error() {
    let input = "let x = 1253.f";
    let mut scanner = Scanner::new(input);
    let err = scanner.scan_tokens().err().unwrap();
    assert!(matches!(
        err,
        Scan(ScanError {
            r#type: InvalidNumber,
            ..
        })
    ));
}

#[test]
fn invalid_expressions_return_error() {
    let cases = [
        ("var x = 1253.f", InvalidNumber),
        (r#"var x = 12. name = "Alphonse""#, InvalidNumber),
        ("v@r pi = 3.415", UnexpectedCharacter),
        (
            r#"var name = "Alphonse; var x = 3.1415"#,
            UnterminatedString,
        ),
    ];
    for (input, _expected_error_type) in cases {
        let mut scanner = Scanner::new(input);
        let err = scanner.scan_tokens().err().unwrap();
        assert!(matches!(
            err,
            Scan(ScanError {
                r#type: _expected_error_type,
                ..
            })
        ));
    }
}
