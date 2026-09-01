mod expr;
mod stmt;

use std::{borrow::Borrow, iter::Peekable};

use crate::lexer::*;
use expr::*;
use stmt::*;

#[derive(Copy, Clone)]
pub enum ParseError<'a>
{
    UnexpectedToken(Token<'a>),
    ExpectedToken {expect: Token<'a>, actual: Token<'a>},
    ExpectedComptName(Token<'a>),
    ExpectedIdentifier(Token<'a>)
}

pub struct Parser<'a>
{
    pub ast: Vec<Stmt<'a>>,
    pub errors: Vec<ParseError<'a>>
}

impl<'a> Parser<'a>
{
    pub fn new() -> Self
    {
        Parser {ast: Vec::new(), errors: Vec::new()}
    }

    pub fn parse<It, T>(&mut self, tok_iter: &mut Peekable<It>)
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(Token::EOF) => return,
            _ => ()
        };

        while !matches!(tok_iter.peek().map(|x| x.borrow()), Some(Token::EOF))
        {
            match Self::parse_stmt(tok_iter)
            {
                Ok(stmt) => self.ast.push(stmt),
                Err(err) => self.errors.push(err)
            }
        }
    }
}