use crate::parser::*;

// Add variant for user-defined type in the future
#[derive(Copy, Clone)]
pub enum ComptType
{
    Int, Float, Bool
}

pub struct EnttType(Vec<ComptType>);

pub enum Stmt<'a>
{
    Decl(EnttType, &'a [u8], Expr<'a>),
    Expr(Expr<'a>)
}

impl<'a> Parser<'a>
{
    // extend to multiple components per entity in the future
    fn parse_entt_type<It, T>(tok_iter: &mut Peekable<It>) -> Result<EnttType, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(Token::RightSqr) => Ok(EnttType(vec![])),
            Some(Token::Int) =>
            {
                tok_iter.next();
                Ok(EnttType(vec![ComptType::Int]))
            },
            Some(Token::Float) =>
            {
                tok_iter.next();
                Ok(EnttType(vec![ComptType::Float]))
            },
            Some(Token::Bool) =>
            {
                tok_iter.next();
                Ok(EnttType(vec![ComptType::Bool]))
            },
            Some(Token::Identifier(_)) => 
            {
                tok_iter.next();
                todo!()
            },
            Some(token) => Err(ParseError::ExpectedComptName(*token)),
            None => unreachable!()
        }
    }

    pub(super) fn parse_stmt<It, T>(tok_iter: &mut Peekable<It>) -> Result<Stmt<'a>, ParseError<'a>>
    where
        It: Iterator<Item = T>,
        T: Borrow<Token<'a>>
    {
        let ret = match tok_iter.peek().map(|x| x.borrow())
        {
            Some(Token::LeftSqr) =>
            {
                tok_iter.next();
                let entt = Self::parse_entt_type(tok_iter)?;
                match tok_iter.peek().map(|x| x.borrow())
                {
                    Some(Token::RightSqr) => tok_iter.next(),
                    Some(token) => return Err(ParseError::UnexpectedToken(*token)),
                    None => unreachable!()
                };
                
                let name;
                match tok_iter.peek().map(|x| x.borrow())
                {
                    Some(Token::Identifier(x)) => 
                    {
                        name = *x;
                        tok_iter.next();
                    },
                    Some(token) => return Err(ParseError::ExpectedIdentifier(*token)),
                    None => unreachable!()
                };

                match tok_iter.peek().map(|x| x.borrow())
                {
                    Some(Token::Eq) => tok_iter.next(),
                    Some(token) => return Err(ParseError::ExpectedToken {expect: Token::Eq, actual: *token}),
                    None => unreachable!()
                };

                Stmt::Decl(entt, name, Self::parse_expr(tok_iter)?)
            },
            Some(_) => Stmt::Expr(Self::parse_expr(tok_iter)?),
            None => unreachable!()
        };

        match tok_iter.peek().map(|x| x.borrow())
        {
            Some(Token::Semicolon) => 
            {
                tok_iter.next();
                Ok(ret)
            },
            Some(token) => Err(ParseError::ExpectedToken {expect: Token::Semicolon, actual: *token}),
            None => unreachable!()
        }
    }
}