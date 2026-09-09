use std::range::Range;

use crate::lexer::*;

// In the future add a variant for custom functions
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Operation
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
pub enum ExprEnum<'s>
{
    Literal(Literal), // -> copy the value into the corresponding component slot in workspace
    Identifier(&'s [u8]), // -> Stack(i64) -> copy this entity into workspace
    Op(Operation, Vec<Expr<'s>>) // -> for each expression, evaluate it, create a new entity, push its id onto the call stack and copy workspace into entity. Then evaluate the return value of function
}

#[derive(PartialEq, Debug)]
pub struct Expr<'s>
{
    pub span: Range<usize>,
    pub data: ExprEnum<'s>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AtomType
{
    Int, Float, Bool
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum VarType
{
    Atom(AtomType),
    Unit
}

#[derive(PartialEq, Debug)]
pub struct StmtBlock<'s>(pub Vec<Stmt<'s>>);

#[derive(PartialEq, Debug)]
pub struct IfElseBlock<'s>
{
    pub cond: Expr<'s>,
    pub block: StmtBlock<'s>,
    pub els: ElseBlock<'s>
}

#[derive(PartialEq, Debug)]
pub enum ElseBlock<'s>
{
    None,
    Else(StmtBlock<'s>),
    ElseIf(Box<IfElseBlock<'s>>)
}

#[derive(PartialEq, Debug)]
pub enum StmtEnum<'s>
{
    Decl(VarType, &'s [u8], Expr<'s>), // create new entity, push its id to stack, evaluate expression, copy workspace to entity
    Expr(Expr<'s>), // -> evaluate
    Block(StmtBlock<'s>),
    Cond(IfElseBlock<'s>),
    Error
}

#[derive(PartialEq, Debug)]
pub struct Stmt<'s>
{
    pub span: Range<usize>,
    pub data: StmtEnum<'s>
}