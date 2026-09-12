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
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Let => write!(f, "let"),
            Token::Fn => write!(f, "fn"),
            
            // Identifiers
            Token::Identifier(bytes) => {
                // The regex only allows ASCII characters, so this unwrap is 100% safe.
                // If you want to be paranoid, use str::from_utf8(bytes)
                let s = std::str::from_utf8(bytes).unwrap();
                write!(f, "{}", s)
            }
            
            // Literals
            Token::Literal(lit) => write!(f, "{}", lit),
            Token::Error => write!(f, "<error>")
        }
    }
}

impl<'s> fmt::Display for Operation<'s>
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
            Operation::Func(iden) => str::from_utf8(iden).unwrap()
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
            ExprEnum::Identifier(id) => write!(f, "{}", str::from_utf8(id).unwrap()),
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
            StmtEnum::Decl(typ, id, expr) => write!(f, "let {}: {} = {};", str::from_utf8(id).unwrap(), typ, expr.data),
            StmtEnum::Expr(expr) => write!(f, "{};", expr.data),
            StmtEnum::Block(block) => write!(f, "{}", block),
            StmtEnum::Cond(if_else) => write!(f, "{}", if_else),
            StmtEnum::Iter(cond, block) => write!(f, "while {} {}", cond.data, block),
            StmtEnum::Break => write!(f, "break;"),
            StmtEnum::Continue => write!(f, "continue;"),
            StmtEnum::FnDecl(id, params, out_typ, block) =>
            {
                write!(f, "{}(", str::from_utf8(id).unwrap())?;
                for param in params {write!(f, "{}: {}, ", str::from_utf8(param.0).unwrap(), param.1)?}
                write!(f, "): {} {}", out_typ, block)
            },
            StmtEnum::Error => write!(f, "<Error>;")
        }
    }
}

fn unres_op<'s>(op: &ResOp) -> Operation<'s>
{
    match op
    {
        ResOp::Negate => Operation::Negate,
        ResOp::Not => Operation::Not,
        ResOp::Print => Operation::Print,
        ResOp::Add => Operation::Add,
        ResOp::Sub => Operation::Sub,
        ResOp::Mul => Operation::Mul,
        ResOp::Div => Operation::Div,
        ResOp::EqualTo => Operation::EqualTo,
        ResOp::NotEqualTo => Operation::NotEqualTo,
        ResOp::Greater => Operation::Greater,
        ResOp::Lesser => Operation::Lesser,
        ResOp::GreaterEq => Operation::GreaterEq,
        ResOp::LesserEq => Operation::LesserEq,
        ResOp::And => Operation::And,
        ResOp::Or => Operation::Or,
        ResOp::Assign => Operation::Assign,
        ResOp::Ternary => Operation::Ternary,
        ResOp::Func(_) => unreachable!()
    }
}

impl<'s> fmt::Display for ResExprEnum
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ResExprEnum::Literal(lit) => write!(f, "{}", lit),
            ResExprEnum::StackBinding(idx) => write!(f, "$env[{}]", idx),
            ResExprEnum::Op(op, args) => 
            {
                match op
                {
                    ResOp::Func(idx) => write!(f, "($fenv[{}]", idx),
                    _ => write!(f, "({}", unres_op(op))
                }?;

                for arg in args {write!(f, " {}", arg.data)?;}
                write!(f, ")")
            }
        }
    }
}

impl<'s> fmt::Display for ResBlock
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
                writeln!(f, "{}", stmt)?;
            }
            write!(f, "}}")
        }
    }
}

impl<'s> fmt::Display for ResIfElse
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "if {} {}{}", self.cond.data, self.block, self.els)
    }
}

impl<'s> fmt::Display for ResElse
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

impl<'s> fmt::Display for ResStmt
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ResStmt::Decl(expr) => write!(f, "decl {};", expr.data),
            ResStmt::Expr(expr) => write!(f, "{};", expr.data),
            ResStmt::Block(res_block) => write!(f, "{}", res_block),
            ResStmt::Cond(if_else) => write!(f, "{}", if_else),
            ResStmt::Iter(cond, block) => write!(f, "while {} {}", cond.data, block),
            ResStmt::Break => write!(f, "break;"),
            ResStmt::Continue => write!(f, "continue;"),
            ResStmt::FnDecl => write!(f, "<function>")
        }
    }
}

pub fn print_res_err(err: ResError, src: &[u8])
{
    match err
    {
        ResError::IdentifierNotFound(span, iden) => 
            println!("Undeclared identifier '{}' at span {}:{}.", str::from_utf8(iden).unwrap(), span.start, span.end),
        ResError::ArgCountMismatch(span) => 
            println!("Invalid number of arguments for function/operator in expression '{}' at span {}:{}.", str::from_utf8(&src[span]).unwrap(), span.start, span.end),
        ResError::TypeMismatch(span, typ) => 
            println!("Expression '{}' of incorrect type '{}' at span {}:{}.", str::from_utf8(&src[span]).unwrap(), typ, span.start, span.end),
        ResError::ExpectedLvalue(span) => 
            println!("Expected lvalue expression, found '{}' at span {}:{}.", str::from_utf8(&src[span]).unwrap(), span.start, span.end),
        ResError::Redecl(span) => 
            println!("Redeclaration of an identifier in statement '{}' at span {}:{}.", str::from_utf8(&src[span]).unwrap(), span.start, span.end),
        ResError::OnlyInLoop(span) =>
            println!("Statement '{}' at span {}:{} can only be used in loops.", str::from_utf8(&src[span]).unwrap(), span.start, span.end)
    }
}