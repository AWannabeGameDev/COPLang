use std::fmt;

use crate::lexer::*;
use crate::ast::*;
use crate::resolver::*;

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

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Punctuation
            Token::LeftSqr => write!(f, "["),
            Token::RightSqr => write!(f, "]"),
            // You have to double up the braces to escape them in format strings
            Token::LeftBrace => write!(f, "{{"), 
            Token::RightBrace => write!(f, "}}"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Question => write!(f, "?"),
            
            // Math Ops
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::ForSlash => write!(f, "/"),
            Token::Star => write!(f, "*"),

            // Logic & Comparison
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::Bang => write!(f, "!"),
            Token::BangEq => write!(f, "!="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEq => write!(f, ">="),
            Token::Lesser => write!(f, "<"),
            Token::LesserEq => write!(f, "<="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),

            // Keywords 
            Token::Int => write!(f, "Int"),
            Token::Float => write!(f, "Float"),
            Token::Bool => write!(f, "Bool"),
            Token::Print => write!(f, "print"),
            Token::Compt => write!(f, "compt"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::Entt => write!(f, "entt"),
            
            // Identifiers
            Token::Identifier(bytes) => {
                // The regex only allows ASCII characters, so this unwrap is 100% safe.
                // If you want to be paranoid, use String::from_utf8_lossy(bytes)
                let s = std::str::from_utf8(bytes).unwrap();
                write!(f, "{}", s)
            }
            
            // Literals
            Token::Literal(lit) => write!(f, "{}", lit),
            Token::Error => write!(f, "<error>")
        }
    }
}

impl fmt::Display for FnName
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let s = match self
        {
            FnName::Negate => "-", FnName::Not => "!", FnName::Print => "print",
            FnName::Add => "+", FnName::Sub => "-", FnName::Mul => "*", FnName::Div => "/",
            FnName::EqualTo => "==", FnName::NotEqualTo => "!=",
            FnName::Greater => ">", FnName::Lesser => "<", 
            FnName::GreaterEq => ">=", FnName::LesserEq => "<=",
            FnName::And => "&&", FnName::Or => "||",
            FnName::Assign => "=", FnName::Ternary => "?:",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for ComptType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let s = match self
        {
            ComptType::Int => "Int",
            ComptType::Float => "Float",
            ComptType::Bool => "Bool",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for EnttType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            EnttType::Compt(c) => write!(f, "{}", c),
            EnttType::Unit => write!(f, "unit"),
        }
    }
}

impl<'a> fmt::Display for Expr<'a>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Identifier(id) => write!(f, "{}", String::from_utf8_lossy(id)),
            Expr::Call(func, args) => 
            {
                write!(f, "({}", func)?;
                for arg in args
                {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl<'a> fmt::Display for Stmt<'a>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Stmt::Decl(typ, id, expr) => write!(f, "entt {}: {} = {};", String::from_utf8_lossy(id), typ, expr),
            Stmt::Expr(expr) => write!(f, "{};", expr),
            Stmt::Error => write!(f, "<Error>;")
        }
    }
}

impl<'a> fmt::Display for AST<'a>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        for stmt in &self.0
        {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

impl fmt::Display for ResExpr
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ResExpr::Literal(lit) => write!(f, "{}", lit),
            ResExpr::StackBinding(idx) => write!(f, "$env[{}]", idx),
            ResExpr::Call(func, args) => 
            {
                write!(f, "({}", func)?;
                for arg in args
                {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for ResStmt
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ResStmt::Decl(expr) => write!(f, "decl {};", expr),
            ResStmt::Expr(expr) => write!(f, "{};", expr),
        }
    }
}

impl fmt::Display for ResAST
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        for stmt in &self.0
        {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

impl<'a, 'b> fmt::Display for ResError<'a, 'b>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ResError::IdentifierNotFound(iden) => 
                write!(f, "Undeclared identifier: '{}'", String::from_utf8_lossy(iden)),
            ResError::ArgCountMismatch(func) => 
                write!(f, "Invalid number of arguments for function/operator '{}'", func),
            ResError::TypeMismatch(expr) => 
                write!(f, "Type mismatch at expression: {}", expr),
            ResError::ExpectedLvalue(expr) => 
                write!(f, "Expected lvalue expression, found: {}", expr),
            ResError::Redecl(iden) => 
                write!(f, "Variable '{}' has already been declared in this scope", String::from_utf8_lossy(iden)),
        }
    }
}