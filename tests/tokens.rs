use claim::{assert_err, assert_ok};
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
#[should_panic(expected = "unterminated string")]
fn unterminated_string_returns_error() {
    let input = r#"var name = "Alphonse"#;
    let mut scanner = Scanner::new(input);
    scanner.scan_tokens().unwrap();
}

#[test]
#[should_panic(expected = "invalid number")]
fn invalid_number_returns_error() {
    let input = "var x = 1253.f";
    let mut scanner = Scanner::new(input);
    scanner.scan_tokens().unwrap();
}

#[test]
fn invalid_expressions_return_error() {
    let cases = [
        "var x = 1253.f",
        r#"var x = 12. name = "Alphonse""#,
        "var pi = 3.l415",
        r#"var name = "Alphonse; var x = 3.1415"#,
    ];
    for input in cases {
        let mut scanner = Scanner::new(input);
        assert_err!(scanner.scan_tokens());
    }
}
