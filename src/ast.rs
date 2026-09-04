use crate::lexer::*;

// In the future add a variant for custom functions
#[derive(Copy, Clone, Debug)]
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

#[derive(Debug)]
pub enum Expr<'a>
{
    Literal(Literal), // -> copy the value into the corresponding component slot in workspace
    Identifier(&'a [u8]), // -> Stack(i64) -> copy this entity into workspace
    Call(FnName, Vec<Expr<'a>>) // -> for each expression, evaluate it, create a new entity, push its id onto the call stack and copy workspace into entity. Then evaluate the return value of function
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ComptType
{
    Int, Float, Bool
}

// TODO: CONVERT TO A TYPE REPO WHICH RETURNS UNIQUE IDS FOR EACH TYPE. AVOIDS CLONING.
#[derive(Clone, PartialEq, Debug)]
pub enum EnttType
{
    Compt(ComptType),
    Unit
}

#[derive(Debug)]
pub enum Stmt<'a>
{
    Decl(EnttType, &'a [u8], Expr<'a>), // create new entity, push its id to stack, evaluate expression, copy workspace to entity
    Expr(Expr<'a>) // -> evaluate
}

#[derive(Debug)]
pub struct AST<'a>(pub Vec<Stmt<'a>>);