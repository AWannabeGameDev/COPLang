// This file is mostly AI generated since I was too lazy to write all the display implemenations 
// (it is busywork with no learning value)

use std::fmt;

use crate::lexer::*;
use crate::ast::*;

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

// --- From stmt.rs ---

impl fmt::Display for ComptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComptType::Int => write!(f, "Int"), //[cite: 3]
            ComptType::Float => write!(f, "Float"), //[cite: 3]
            ComptType::Bool => write!(f, "Bool"), //[cite: 3]
        }
    }
}

impl fmt::Display for EnttType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, compt) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", compt)?;
        }
        write!(f, "]")
    }
}

impl<'a> fmt::Display for Stmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Decl(entt, name, expr) => {
                // Converting the byte slice to a readable string[cite: 3]
                write!(f, "entt {}: {} = {};", String::from_utf8_lossy(name), entt, expr)
            }
            Stmt::Expr(expr) => write!(f, "{};", expr), //[cite: 3]
        }
    }
}

// --- From expr.rs ---

impl fmt::Display for FnName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            FnName::Negate => "-", //[cite: 1]
            FnName::Not => "!", //[cite: 1]
            FnName::Add => "+", //[cite: 1]
            FnName::Sub => "-", //[cite: 1]
            FnName::Mul => "*", //[cite: 1]
            FnName::Div => "/", //[cite: 1]
            FnName::EqualTo => "==", //[cite: 1]
            FnName::NotEqualTo => "!=", //[cite: 1]
            FnName::Greater => ">", //[cite: 1]
            FnName::Lesser => "<", //[cite: 1]
            FnName::GreaterEq => ">=", //[cite: 1]
            FnName::LesserEq => "<=", //[cite: 1]
            FnName::And => "&&", //[cite: 1]
            FnName::Or => "||", //[cite: 1]
            FnName::Assign => "=", //[cite: 1]
            FnName::Ternary => "?:", //[cite: 1]
            FnName::Print => "print"
        };
        write!(f, "{}", op)
    }
}

impl<'a> fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{}", lit), //[cite: 1]
            Expr::Identifier(ident) => write!(f, "{}", String::from_utf8_lossy(ident)), //[cite: 1]
            Expr::Call(func, args) => { //[cite: 1]
                // Using S-expression formatting for a cleaner AST print
                write!(f, "({}", func)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl<'a> fmt::Display for AST<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.0 {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}