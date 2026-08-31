// This file is mostly AI generated since I was too lazy to write all the display implemenations 
// (it is busywork with no learning value)

use std::fmt;

use crate::*;

impl fmt::Display for Literal 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        match self 
        {
            Literal::Int(i) => write!(f, "{}", i),
            Literal::Float(fl) => write!(f, "{}", fl),
            Literal::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl<'a> fmt::Display for Token<'a> 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        // Match the operators that actually show up inside expressions. 
        // For structural tokens (like braces or EOF), we just fall back to the Debug trait.
        match self 
        {
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::ForSlash => write!(f, "/"),
            Token::Star => write!(f, "*"),
            Token::Bang => write!(f, "!"),
            Token::EqEq => write!(f, "=="),
            Token::BangEq => write!(f, "!="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEq => write!(f, ">="),
            Token::Lesser => write!(f, "<"),
            Token::LesserEq => write!(f, "<="),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Eq => write!(f, "="),
            Token::Identifier(bytes) => 
            {
                // Since your identifier is a byte slice, we gotta parse it to a string.
                // If you somehow pass bad UTF-8, it handles it without panicking.
                let s = std::str::from_utf8(bytes).unwrap_or("<invalid_utf8>");
                write!(f, "{}", s)
            },
            Token::Literal(lit) => write!(f, "{}", lit),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negate => write!(f, "-"),
            Self::Not => write!(f, "!"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Greater => write!(f, ">"),
            Self::Lesser => write!(f, "<"), // naming this 'Lesser' instead of 'Less' is a choice, but I respect it XD
            Self::GreaterEq => write!(f, ">="),
            Self::LesserEq => write!(f, "<="),
            Self::DiscardLeft => write!(f, ","),
        }
    }
}

impl fmt::Display for Expr
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        match self 
        {
            Expr::Unary(op, right) => write!(f, "({} {right})", op),
            Expr::Binary(left, op, right) => write!(f, "({left} {} {right})", op),
            Expr::Ternary(cond, true_expr, false_expr) => write!(f, "({cond} ? {true_expr} : {false_expr})"),
            Expr::Literal(lit) => write!(f, "{lit}"),
            Expr::EOF => write!(f, "EOF"),
        }
    }
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NoClosingParen {after} => {
                write!(f, "Missing closing parenthesis after {after}")
            }
            ParseError::UnexpectedEOF => {
                write!(f, "Unexpected end of file while parsing")
            }
            ParseError::UnexpectedToken(token) => {
                // Now this just hooks directly into FullToken's Display impl
                write!(f, "Unexpected token: {token}")
            },
            ParseError::MissingLeftOperand {op} =>
            {
                write!(f, "Missing left operand for {}", op)
            }
        }
    }
}