use std::{borrow::Borrow, iter::Peekable};

use crate::parser::*;

// In the future add a variant for custom functions
#[derive(Copy, Clone)]
pub enum FnName
{
    Negate, Not, Print,
    Add, Sub, Mul, Div,
    EqualTo, NotEqualTo,
    Greater, Lesser, GreaterEq, LesserEq,
    And, Or,
    DiscardLeft,
    Assign,
    Ternary
}

fn unary_op(tok: Token) -> FnName
{
    assert!(matches!(tok, Token::Minus | Token::Bang | Token::Print));
        
    match tok
    {
        Token::Minus => FnName::Negate,
        Token::Bang => FnName::Not,
        Token::Print => FnName::Print,
        _ => unreachable!()
    }
}

fn binary_op(tok: Token) -> FnName
{
    assert!(matches!(tok, Token::Plus | Token::Minus | Token::Star | Token::ForSlash |
        Token::EqEq | Token::BangEq |
        Token::Greater | Token::Lesser | Token::GreaterEq | Token::LesserEq | Token::And | Token::Or |
        Token::Comma |
        Token::Eq));

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
        Token::Eq => FnName::Assign,
        _ => unreachable!()
    }
}

pub enum Expr<'a>
{
    Literal(Literal),
    Identifier(&'a [u8]),
    Call(FnName, Vec<Expr<'a>>)
}

macro_rules! left_binary_op
{
    ($vis:vis, $name:ident, $next:ident, $pat:pat) =>
    {
        $vis fn $name<It, T>(tok_iter: &mut Peekable<It>) -> Result<Expr<'a>, ParseError<'a>>
        where
            It: Iterator<Item = T>,
            T: Borrow<Token<'a>>
        {
            let mut ret = Self::$next(tok_iter)?;

            while let Some(token) = tok_iter.peek().map(|x| x.borrow())
            {
                match token
                {
                    $pat =>
                    {
                        let op = *token;
                        tok_iter.next();
                        ret = Expr::Call(binary_op(op), vec![ret, Self::$next(tok_iter)?])
                    },
                    _ => break
                }
            }

            Ok(ret)
        }
    }
}

impl<'a> Parser<'a>
{
    left_binary_op!(pub(super), parse_expr, parse_assign, Token::Comma);

    fn parse_assign<It, T>(tok_iter: &mut Peekable<It>) -> Result<Expr<'a>, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        let mut ret = Self::parse_ternary(tok_iter)?;
        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(Token::Eq) =>
            {
                tok_iter.next();
                ret = Expr::Call(FnName::Assign, vec![ret, Self::parse_assign(tok_iter)?])
            },
            _ => ()
        }

        Ok(ret)
    }

    fn parse_ternary<It, T>(tok_iter: &mut Peekable<It>) -> Result<Expr<'a>, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        let mut ret = Self::parse_logic(tok_iter)?;

        while let Some(token) = tok_iter.peek().map(|x| x.borrow())
        {
            match token
            {
                Token::Question =>
                {
                    tok_iter.next();
                    let if_branch = Self::parse_ternary(tok_iter)?;

                    match tok_iter.peek().map(|x| x.borrow())
                    {
                        Some(Token::Colon) =>
                        {
                            tok_iter.next();
                            ret = Expr::Call(FnName::Ternary, vec![ret, if_branch, Self::parse_ternary(tok_iter)?]);
                            break
                        },
                        Some(token) => return Err(ParseError::ExpectedToken {expect: Token::Colon, actual: *token}),
                        None => unreachable!()
                    }
                },
                _ => break
            }
        }

        Ok(ret)
    }

    left_binary_op!(, parse_logic, parse_eq, Token::And | Token::Or);
    left_binary_op!(, parse_eq, parse_ineq, Token::EqEq | Token::BangEq);
    left_binary_op!(, parse_ineq, parse_term, Token::Greater | Token::Lesser | Token::GreaterEq | Token::LesserEq);
    left_binary_op!(, parse_term, parse_factor, Token::Plus | Token::Minus);
    left_binary_op!(, parse_factor, parse_unary, Token::Star | Token::ForSlash);

    fn parse_unary<It, T>(tok_iter: &mut Peekable<It>) -> Result<Expr<'a>, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(op @ (Token::Minus | Token::Bang | Token::Print)) => 
            {
                let op = *op;
                tok_iter.next();
                Ok(Expr::Call(unary_op(op), vec![Self::parse_unary(tok_iter)?]))
            },
            _ => Self::parse_final(tok_iter)
        }
    }

    // In the future, add parsing for identifiers and function calls
    fn parse_final<It, T>(tok_iter: &mut Peekable<It>) -> Result<Expr<'a>, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(Token::LeftParen) =>
            {
                tok_iter.next();
                let ret = Self::parse_expr(tok_iter)?;

                match tok_iter.peek().map(|x| x.borrow())
                {
                    Some(Token::RightParen) =>
                    {
                        tok_iter.next();
                        Ok(ret)
                    },
                    Some(token) => Err(ParseError::ExpectedToken {expect: Token::RightParen, actual: *token}),
                    None => unreachable!()
                }
            },
            Some(Token::Literal(x)) =>
            {
                let x = *x;
                tok_iter.next();
                Ok(Expr::Literal(x))
            },
            Some(Token::Identifier(x)) =>
            {
                // add support for function call syntax in the future
                let x = *x;
                tok_iter.next();
                Ok(Expr::Identifier(x))
            },
            Some(token) =>
            {
                Err(ParseError::UnexpectedToken(*token))
            },
            None => unreachable!()
        }
    }
}