use rlox::expression::Expression::*;
use rlox::expression::Object::*;
use rlox::scanner::{Token, TokenType};

#[test]
fn check_expr_repr() {
    let expr1 = Binary {
        left: Box::new(Unary {
            op: Token {
                r#type: TokenType::Minus,
                lexeme: "-".to_string(),
                line: 0,
                col: 0,
            },
            right: Box::new(Literal(Number(3.14))),
        }),
        op: Token {
            r#type: TokenType::Star,
            lexeme: "*".to_string(),
            line: 0,
            col: 0,
        },
        right: Box::new(Grouping(Box::new(Literal(Number(3.151))))),
    };
    let op = Token {
        r#type: TokenType::EqualEqual,
        lexeme: "==".to_string(),
        line: 0,
        col: 0,
    };
    let expr2 = Unary {
        op: Token {
            r#type: TokenType::Minus,
            lexeme: "-".to_string(),
            line: 0,
            col: 0,
        },
        right: Box::new(Literal(Number(9.89))),
    };
    let expr = Binary {
        left: Box::new(expr1),
        op,
        right: Box::new(expr2),
    };
    assert_eq!(expr.to_string(), "(== (* (- 3.14) (group 3.151)) (- 9.89))");
}
