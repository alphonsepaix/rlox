use rlox::grammar::*;
use rlox::scanner::{Token, TokenType};

#[test]
fn check_expr_repr() {
    let expr1 = Expression::Binary {
        left: Box::new(Expression::Unary {
            op: Token {
                r#type: TokenType::Minus,
                lexeme: "-".to_string(),
                line: 0,
                col: 0,
            },
            right: Box::new(Expression::Literal(Object::Number(3.14))),
        }),
        op: Token {
            r#type: TokenType::Star,
            lexeme: "*".to_string(),
            line: 0,
            col: 0,
        },
        right: Box::new(Expression::Grouping(Box::new(Expression::Literal(
            Object::Number(3.151),
        )))),
    };
    let op = Token {
        r#type: TokenType::EqualEqual,
        lexeme: "==".to_string(),
        line: 0,
        col: 0,
    };
    let expr2 = Expression::Unary {
        op: Token {
            r#type: TokenType::Minus,
            lexeme: "-".to_string(),
            line: 0,
            col: 0,
        },
        right: Box::new(Expression::Literal(Object::Number(9.89))),
    };
    let expr = Expression::Binary {
        left: Box::new(expr1),
        op,
        right: Box::new(expr2),
    };
    assert_eq!(expr.repr(), "(== (* (- 3.14) (group 3.151)) (- 9.89))");
}

#[test]
fn check_expr_rpn() {
    let expr = Expression::Binary {
        left: Box::new(Expression::Unary {
            op: Token {
                r#type: TokenType::Minus,
                lexeme: "-".to_string(),
                line: 0,
                col: 0,
            },
            right: Box::new(Expression::Literal(Object::Number(3.14))),
        }),
        op: Token {
            r#type: TokenType::Star,
            lexeme: "*".to_string(),
            line: 0,
            col: 0,
        },
        right: Box::new(Expression::Grouping(Box::new(Expression::Literal(
            Object::Number(3.151),
        )))),
    };
    assert_eq!(expr.rpn(), "-3.14 3.151 *");

    let expr = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Object::Number(1.0))),
            op: Token {
                r#type: TokenType::Plus,
                lexeme: "+".to_string(),
                line: 0,
                col: 0,
            },
            right: Box::new(Expression::Literal(Object::Number(2.0))),
        }),
        op: Token {
            r#type: TokenType::Star,
            lexeme: "*".to_string(),
            line: 0,
            col: 0,
        },
        right: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Object::Number(4.0))),
            op: Token {
                r#type: TokenType::Minus,
                lexeme: "-".to_string(),
                line: 0,
                col: 0,
            },
            right: Box::new(Expression::Literal(Object::Number(3.0))),
        }),
    };
    assert_eq!(expr.rpn(), "1 2 + 4 3 - *");
}
