use std::range::Range;

use crate::lexer::*;

// In the future add a variant for custom functions
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum FnName
{
    Negate, Not, Print,
    Add, Sub, Mul, Div,
    EqualTo, NotEqualTo,
    Greater, Lesser, GreaterEq, LesserEq,
    And, Or,
    Assign,
    Ternary
}

#[derive(PartialEq, Debug)]
pub enum ExprEnum<'a>
{
    Literal(Literal), // -> copy the value into the corresponding component slot in workspace
    Identifier(&'a [u8]), // -> Stack(i64) -> copy this entity into workspace
    Call(FnName, Vec<Expr<'a>>) // -> for each expression, evaluate it, create a new entity, push its id onto the call stack and copy workspace into entity. Then evaluate the return value of function
}

#[derive(PartialEq, Debug)]
pub struct Expr<'a>
{
    pub span: Range<usize>,
    pub data: ExprEnum<'a>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ComptType
{
    Int, Float, Bool
}

#[derive(Clone, PartialEq, Debug)]
pub enum EnttType
{
    Compt(ComptType),
    Unit
}

#[derive(PartialEq, Debug)]
pub enum StmtEnum<'a>
{
    Decl(EnttType, &'a [u8], Expr<'a>), // create new entity, push its id to stack, evaluate expression, copy workspace to entity
    Expr(Expr<'a>), // -> evaluate
    Error
}

#[derive(PartialEq, Debug)]
pub struct Stmt<'a>
{
    pub span: Range<usize>,
    pub data: StmtEnum<'a>
}

#[derive(Debug)]
pub struct AST<'a>(pub Vec<Stmt<'a>>);