use std::{borrow::Borrow, iter::Peekable};

use crate::lexer::*;

// In the future add a variant for custom functions
#[derive(Copy, Clone)]
pub enum FnName
{
    Negate, Not,
    Add, Sub, Mul, Div,
    EqualTo, NotEqualTo,
    Greater, Lesser, GreaterEq, LesserEq,
    And, Or,
    DiscardLeft,
    Ternary
}

fn unary_op(tok: Token) -> FnName
{
    assert!(matches!(tok, Token::Minus | Token::Bang));
        
    match tok
    {
        Token::Minus => FnName::Negate,
        Token::Bang => FnName::Not,
        _ => unreachable!()
    }
}

fn binary_op(tok: Token) -> FnName
{
    assert!(matches!(tok, Token::Plus | Token::Minus | Token::Star | Token::ForSlash |
        Token::And | Token::Or |
        Token::EqEq | Token::BangEq |
        Token::Greater | Token::Lesser | Token::GreaterEq | Token::LesserEq | Token::Comma));

    match tok
    {
        Token::Plus => FnName::Add,
        Token::Minus => FnName::Sub,
        Token::Star => FnName::Mul,
        Token::ForSlash => FnName::Div,
        Token::Greater => FnName::Greater,
        Token::Lesser => FnName::Lesser,
        Token::GreaterEq => FnName::GreaterEq,
        Token::LesserEq => FnName::LesserEq,
        Token::Comma => FnName::DiscardLeft,
        Token::And => FnName::And,
        Token::Or => FnName::Or,
        Token::EqEq => FnName::EqualTo,
        Token::BangEq => FnName::NotEqualTo,
        _ => unreachable!()
    }
}

pub enum Expr
{
    Literal(Literal),
    Call(FnName, Vec<Expr>),
    Empty
}

pub struct Parser<'a>
{
    pub ast: Expr,
    pub errors: Vec<ParseError<'a>>
}

#[derive(Debug, Copy, Clone)]
pub enum ParseError<'a>
{
    NoClosingParen {after: Token<'a>},
    UnexpectedEOF,
    UnexpectedToken(Token<'a>),
    MissingLeftOperand {op: Token<'a>}
}

macro_rules! all_binary
{
    () => 
    {
        Token::EqEq | Token::BangEq |
        Token::Greater | Token::Lesser | Token::GreaterEq | Token::LesserEq |
        Token::Plus | Token::Minus |
        Token::Star | Token::ForSlash
    }
}

macro_rules! binary_op
{
    ($name:ident, $next:ident, $pat:pat) =>
    {
        fn $name<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Expr, ParseError<'a>>
        where
            It: Iterator<Item = T>,
            T: Borrow<Token<'a>>
        {
            let mut ret = self.$next(tok_iter);
            while let Err(err @ ParseError::MissingLeftOperand{op: $pat}) = ret
            {
                tok_iter.next();
                ret = self.$next(tok_iter);
                self.errors.push(err);
            }
            let mut ret = ret?;

            while let Some(token) = tok_iter.peek().map(|x| x.borrow())
            {
                match token
                {
                    $pat =>
                    {
                        let op = *token;
                        tok_iter.next();

                        let mut next = self.$next(tok_iter);
                        while let Err(err @ ParseError::MissingLeftOperand{op: $pat}) = next
                        {
                            tok_iter.next();
                            next = self.$next(tok_iter);
                            self.errors.push(err);
                        }

                        ret = Expr::Call(binary_op(op), vec![ret, next?]);
                    },
                    _ => {break;}
                }
            }

            Ok(ret)
        }
    }
}

impl<'a> Parser<'a>
{
    pub fn new() -> Self
    {
        Parser {ast: Expr::Empty, errors: Vec::new()}
    }

    pub fn parse<It, T>(&mut self, tok_iter: &mut Peekable<It>)
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        let expr = self.parse_expr(tok_iter);
        match expr
        {
            Ok(res) => if !matches!(res, Expr::Empty) {self.ast = res},
            Err(err) => self.errors.push(err)
        }
    }

    fn parse_expr<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Expr, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        let ret = self.parse_comma(tok_iter)?;
        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(token) => Err(ParseError::UnexpectedToken(*token)),
            None => Ok(ret)
        }
    }

    binary_op!(parse_comma, parse_ternary, Token::Comma);

    fn parse_ternary<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Expr, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        let mut ret = self.parse_logic(tok_iter)?;

        while let Some(token) = tok_iter.peek().map(|x| x.borrow())
        {
            match token
            {
                Token::Question =>
                {
                    tok_iter.next();
                    let if_branch = self.parse_ternary(tok_iter)?;

                    match tok_iter.peek().map(|x| x.borrow())
                    {
                        Some(Token::Colon) =>
                        {
                            tok_iter.next();
                            ret = Expr::Call(FnName::Ternary, vec![ret, if_branch, self.parse_ternary(tok_iter)?]);
                            break;
                        },
                        Some(token) => {return Err(ParseError::UnexpectedToken(*token))},
                        None => {return Err(ParseError::UnexpectedEOF);}
                    }
                },
                _ => {break;}
            }
        }

        Ok(ret)
    }

    binary_op!(parse_logic, parse_eq, Token::And | Token::Or);
    binary_op!(parse_eq, parse_relation, Token::EqEq | Token::BangEq);
    binary_op!(parse_relation, parse_term, Token::Greater | Token::Lesser | Token::GreaterEq | Token::LesserEq);
    binary_op!(parse_term, parse_factor, Token::Plus | Token::Minus);
    binary_op!(parse_factor, parse_unary, Token::Star | Token::ForSlash);

    fn parse_unary<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Expr, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(token @ (Token::Minus | Token::Bang)) => 
            {
                let token = *token;
                tok_iter.next();
                Ok(Expr::Call(unary_op(token), vec![self.parse_unary(tok_iter)?]))
            },
            Some(_) => self.parse_final(tok_iter),
            None => Ok(Expr::Empty)
        }
    }

    // In the future, add parsing for identifiers and function calls
    fn parse_final<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Expr, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(Token::LeftParen) =>
            {
                tok_iter.next();
                let ret = self.parse_comma(tok_iter)?;

                match tok_iter.peek().map(|x| x.borrow())
                {
                    Some(Token::RightParen) =>
                    {
                        tok_iter.next();
                        Ok(ret)
                    },
                    Some(token) => Err(ParseError::NoClosingParen {after: *token}),
                    None => Err(ParseError::UnexpectedEOF)
                }
            },
            Some(Token::Literal(x)) =>
            {
                let x = *x;
                tok_iter.next();
                Ok(Expr::Literal(x))
            },
            Some(op @ all_binary!()) =>
            {
                Err(ParseError::MissingLeftOperand {op: *op})
            },
            Some(token) =>
            {
                Err(ParseError::UnexpectedToken(*token))
            },
            None => 
            {
                Ok(Expr::Empty)
            }
        }
    }
}