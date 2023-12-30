use claim::assert_ok;
use rlox::scanner::*;

#[test]
fn simple_expression_tokenized_correctly() {
    let input = r#"var name = "Alphonse""#;
    let mut scanner = Scanner::new(input);

    assert_ok!(scanner.scan_tokens());

    let tokens = vec![
        Token {
            r#type: TokenType::Var,
            lexeme: "var".to_string(),
            line: 1,
            col: 1,
        },
        Token {
            r#type: TokenType::Identifier("name".to_string()),
            lexeme: "name".to_string(),
            line: 1,
            col: 5,
        },
        Token {
            r#type: TokenType::Equal,
            lexeme: "=".to_string(),
            line: 1,
            col: 10,
        },
        Token {
            r#type: TokenType::String("Alphonse".to_string()),
            lexeme: "\"Alphonse\"".to_string(),
            line: 1,
            col: 12,
        },
        Token {
            r#type: TokenType::Eof,
            lexeme: "".to_string(),
            line: 1,
            col: input.len() + 1,
        },
    ];

    assert_eq!(scanner.tokens(), &tokens);
}

#[test]
fn unterminated_string_returns_error() {
    let input = r#"var name = "Alphonse"#;
    let mut scanner = Scanner::new(input);
    let err = scanner.scan_tokens().err().unwrap();
    assert_eq!(err.r#type, ErrorType::UnterminatedString);
}

#[test]
fn invalid_number_returns_error() {
    let input = "var x = 1253.f";
    let mut scanner = Scanner::new(input);
    let err = scanner.scan_tokens().err().unwrap();
    assert_eq!(err.r#type, ErrorType::InvalidNumber);
}

#[test]
fn invalid_expressions_return_error() {
    let cases = [
        ("var x = 1253.f", ErrorType::InvalidNumber),
        (r#"var x = 12. name = "Alphonse""#, ErrorType::InvalidNumber),
        ("v@r pi = 3.415", ErrorType::UnexpectedCharacter),
        (
            r#"var name = "Alphonse; var x = 3.1415"#,
            ErrorType::UnterminatedString,
        ),
    ];
    for (input, expected_error_type) in cases {
        let mut scanner = Scanner::new(input);
        let err = scanner.scan_tokens().err().unwrap();
        assert_eq!(err.r#type, expected_error_type);
    }
}
