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
            Token::Struct => write!(f, "struct"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::Let => write!(f, "let"),
            
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

impl fmt::Display for Operation
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let s = match self
        {
            Operation::Negate => "-", Operation::Not => "!", Operation::Print => "print",
            Operation::Add => "+", Operation::Sub => "-", Operation::Mul => "*", Operation::Div => "/",
            Operation::EqualTo => "==", Operation::NotEqualTo => "!=",
            Operation::Greater => ">", Operation::Lesser => "<", 
            Operation::GreaterEq => ">=", Operation::LesserEq => "<=",
            Operation::And => "&&", Operation::Or => "||",
            Operation::Assign => "=", Operation::Ternary => "?:",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for AtomType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let s = match self
        {
            AtomType::Int => "Int",
            AtomType::Float => "Float",
            AtomType::Bool => "Bool",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for VarType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            VarType::Atom(c) => write!(f, "{}", c),
            VarType::Unit => write!(f, "unit"),
        }
    }
}

impl<'s> fmt::Display for ExprEnum<'s>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ExprEnum::Literal(lit) => write!(f, "{}", lit),
            ExprEnum::Identifier(id) => write!(f, "{}", String::from_utf8_lossy(id)),
            ExprEnum::Op(op, args) => 
            {
                write!(f, "({}", op)?;
                for arg in args
                {
                    write!(f, " {}", arg.data)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl<'s> fmt::Display for StmtBlock<'s>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if self.0.is_empty()
        {
            write!(f, "{{}}")
        }
        else
        {
            writeln!(f, "{{")?;
            for stmt in &self.0
            {
                writeln!(f, "{}", stmt.data)?;
            }
            write!(f, "}}")
        }
    }
}

impl<'s> fmt::Display for IfElseBlock<'s>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "if {} {}{}", self.cond.data, self.block, self.els)
    }
}

impl<'s> fmt::Display for ElseBlock<'s>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ElseBlock::None => Ok(()),
            ElseBlock::Else(block) => write!(f, " else {}", block),
            ElseBlock::ElseIf(if_else) => write!(f, " else {}", if_else),
        }
    }
}

impl<'s> fmt::Display for StmtEnum<'s>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            StmtEnum::Decl(typ, id, expr) => write!(f, "let {}: {} = {};", String::from_utf8_lossy(id), typ, expr.data),
            StmtEnum::Expr(expr) => write!(f, "{};", expr.data),
            StmtEnum::Block(block) => write!(f, "{}", block),
            StmtEnum::Cond(if_else) => write!(f, "{}", if_else),
            StmtEnum::Error => write!(f, "<Error>;"),
        }
    }
}

impl fmt::Display for ResExprEnum
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ResExprEnum::Literal(lit) => write!(f, "{}", lit),
            ResExprEnum::StackBinding(idx) => write!(f, "$env[{}]", idx),
            ResExprEnum::Op(op, args) => 
            {
                write!(f, "({}", op)?;
                for arg in args
                {
                    write!(f, " {}", arg.data)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for ResBlock
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if self.stmts.is_empty()
        {
            write!(f, "{{}}")
        }
        else
        {
            writeln!(f, "{{")?;
            for stmt in &self.stmts
            {
                writeln!(f, "{}", stmt)?;
            }
            write!(f, "}}")
        }
    }
}

impl fmt::Display for ResIfElse
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "if {} {}{}", self.cond.data, self.block, self.els)
    }
}

impl fmt::Display for ResElse
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ResElse::None => Ok(()),
            ResElse::Else(block) => write!(f, " else {}", block),
            ResElse::ElseIf(if_else) => write!(f, " else {}", if_else),
        }
    }
}

impl fmt::Display for ResStmt
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ResStmt::Decl(expr) => write!(f, "decl {};", expr.data),
            ResStmt::Expr(expr) => write!(f, "{};", expr.data),
            ResStmt::Block(res_block) => write!(f, "{}", res_block),
            ResStmt::Cond(if_else) => write!(f, "{}", if_else),
        }
    }
}

pub fn print_res_err(err: ResError, src: &[u8])
{
    match err
    {
        ResError::IdentifierNotFound(span, iden) => 
            println!("Undeclared identifier '{}' at span {}:{}.", String::from_utf8_lossy(iden), span.start, span.end),
        ResError::ArgCountMismatch(span) => 
            println!("Invalid number of arguments for function/operator in expression '{}' at span {}:{}.", String::from_utf8_lossy(&src[span]), span.start, span.end),
        ResError::TypeMismatch(span, typ) => 
            println!("Expression '{}' of incorrect type '{}' at span {}:{}.", String::from_utf8_lossy(&src[span]), typ, span.start, span.end),
        ResError::ExpectedLvalue(span) => 
            println!("Expected lvalue expression, found '{}' at span {}:{}.", String::from_utf8_lossy(&src[span]), span.start, span.end),
        ResError::Redecl(span) => 
            println!("Redeclaration of an identifier in statement '{}' at span {}:{}.", String::from_utf8_lossy(&src[span]), span.start, span.end),
    }
}