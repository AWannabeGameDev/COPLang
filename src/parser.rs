use std::{borrow::Borrow, iter::Peekable};

use crate::lexer::*;

#[derive(Copy, Clone, Debug)]
pub enum UnaryOp
{
    Negate,
    Not
}

fn unary_op(tok: Token) -> UnaryOp
{
    assert!(matches!(tok, Token::Minus | Token::Bang));
        
    match tok
    {
        Token::Minus => UnaryOp::Negate,
        Token::Bang => UnaryOp::Not,
        _ => panic!()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BinaryOp
{
    Add, Sub, Mul, Div,
    Greater, Lesser, GreaterEq, LesserEq,
    DiscardLeft
}


fn binary_op(tok: Token) -> BinaryOp
{
    assert!(matches!(tok, Token::Plus | Token::Minus | Token::Star | Token::ForSlash | 
        Token::Greater | Token::Lesser | Token::GreaterEq | Token::LesserEq | Token::Comma));

    match tok
    {
        Token::Plus => BinaryOp::Add,
        Token::Minus => BinaryOp::Sub,
        Token::Star => BinaryOp::Mul,
        Token::ForSlash => BinaryOp::Div,
        Token::Greater => BinaryOp::Greater,
        Token::Lesser => BinaryOp::Lesser,
        Token::GreaterEq => BinaryOp::GreaterEq,
        Token::LesserEq => BinaryOp::LesserEq,
        Token::Comma => BinaryOp::DiscardLeft,
        _ => panic!()
    }
}

#[derive(Debug)]
pub enum Expr
{
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Literal(Literal),
    EOF
}

pub struct Parser<'a>
{
    pub ast: Option<Box<Expr>>,
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
        fn $name<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Box<Expr>, ParseError<'a>>
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

                        ret = Box::new(Expr::Binary(ret, binary_op(op), next?));
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
        Parser {ast: None, errors: Vec::new()}
    }

    pub fn parse<It, T>(&mut self, tok_iter: &mut Peekable<It>)
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        let expr = self.parse_expr(tok_iter);
        match expr
        {
            Ok(res) => self.ast = Some(res),
            Err(err) => self.errors.push(err),
        }
    }

    fn parse_expr<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Box<Expr>, ParseError<'a>>
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

    fn parse_ternary<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Box<Expr>, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        let mut ret = self.parse_eq(tok_iter)?;

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
                            ret = Box::new(Expr::Ternary(ret, if_branch, self.parse_ternary(tok_iter)?));
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

    binary_op!(parse_eq, parse_relation, Token::EqEq | Token::BangEq);
    binary_op!(parse_relation, parse_term, Token::Greater | Token::Lesser | Token::GreaterEq | Token::LesserEq);
    binary_op!(parse_term, parse_factor, Token::Plus | Token::Minus);
    binary_op!(parse_factor, parse_unary, Token::Star | Token::ForSlash);

    fn parse_unary<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Box<Expr>, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(token @ Token::Minus) => 
            {
                let token = *token;
                tok_iter.next();
                Ok(Box::new(Expr::Unary(unary_op(token), self.parse_unary(tok_iter)?)))
            },
            Some(_) => self.parse_final(tok_iter),
            None => Ok(Box::new(Expr::EOF))
        }
    }

    fn parse_final<It, T>(&mut self, tok_iter: &mut Peekable<It>) -> Result<Box<Expr>, ParseError<'a>>
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
                Ok(Box::new(Expr::Literal(x)))
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
                Ok(Box::new(Expr::EOF))
            }
        }
    }
}