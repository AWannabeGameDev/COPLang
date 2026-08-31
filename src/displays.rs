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

impl fmt::Display for FnName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            FnName::Negate => "-",
            FnName::Not => "!",
            FnName::DiscardLeft => ",",
            FnName::Add => "+",
            FnName::Sub => "-",
            FnName::Mul => "*",
            FnName::Div => "/",
            FnName::Greater => ">",
            FnName::Lesser => "<",
            FnName::GreaterEq => ">=",
            FnName::LesserEq => "<=",
            FnName::And => "&&",
            FnName::Or => "||",
            FnName::Ternary => "?:",
            FnName::EqualTo => "==",
            FnName::NotEqualTo => "!=",
        };
        write!(f, "{op}")
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{lit}"),
            Expr::Empty => write!(f, "()"), // or whatever represents an empty node in your lang
            Expr::Call(op, args) => {
                // Formatting as (operator arg1 arg2) 
                write!(f, "({op}")?;
                for arg in args {
                    write!(f, " {arg}")?;
                }
                write!(f, ")")
            }
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

impl fmt::Display for ExprType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ExprType::Int => write!(f, "Int"),
            ExprType::Float => write!(f, "Float"),
            ExprType::Bool => write!(f, "Bool"),
            ExprType::Unit => write!(f, "Unit"),
        }
    }
}

impl fmt::Display for HalfResExpr
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            HalfResExpr::Literal(lit) => write!(f, "{}", lit),
            HalfResExpr::Call(fn_name, args) =>
            {
                write!(f, "{}(", fn_name)?;
                for (i, arg) in args.iter().enumerate()
                {
                    if i > 0 {write!(f, ", ")?;}
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            HalfResExpr::Empty => write!(f, "<empty>"),
        }
    }
}

impl fmt::Display for ResExpr
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        // Formats like a proper typed AST: (5 + 3 : Int)
        write!(f, "({}:{})", self.expr, self.typ)
    }
}

impl fmt::Display for TypeError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            TypeError::ArgCountMismatch(name) => write!(f, "Argument count mismatch for function '{}'", name),
            TypeError::ArgTypeMismatch(name) => write!(f, "Argument type mismatch for function '{}'", name),
        }
    }
}

impl fmt::Display for ExprResult
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ExprResult::Literal(x) => write!(f, "{}", x),
            ExprResult::Unit => write!(f, "()"),
        }
    }
}

impl fmt::Display for RuntimeError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            RuntimeError::DivByZero(name) => write!(f, "Division by zero in args to function '{}'", name),
        }
    }
}