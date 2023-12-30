use rlox::grammar::*;
use rlox::scanner::{Token, TokenType};

#[test]
fn check_expr_representation() {
    let expr1 = Binary::new(
        Unary::new(
            Token {
                r#type: TokenType::Minus,
                lexeme: "-".to_string(),
                line: 0,
                col: 0,
            },
            Literal::new(Object::Number(3.14)),
        ),
        Token {
            r#type: TokenType::Star,
            lexeme: "*".to_string(),
            line: 0,
            col: 0,
        },
        Grouping::new(Literal::new(Object::Number(3.151))),
    );
    let op = Token {
        r#type: TokenType::EqualEqual,
        lexeme: "==".to_string(),
        line: 0,
        col: 0,
    };
    let expr2 = Unary::new(
        Token {
            r#type: TokenType::Minus,
            lexeme: "-".to_string(),
            line: 0,
            col: 0,
        },
        Literal::new(Object::Number(9.89)),
    );
    let expr = Binary::new(expr1, op, expr2);
    let repr = expr.to_string();
    assert_eq!(repr, "(== (* (- 3.14) (group 3.151)) (- 9.89))");
}
