use std::collections::HashSet;
use std::str;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Literal
{
    Int(i64), 
    Float(f64), 
    Bool(bool)
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Token<'a>
{
    LeftSqr, RightSqr, LeftBrace, RightBrace, LeftParen, RightParen, Semicolon, Colon, Comma, Dot, Question, 
    Plus, Minus, ForSlash, Star,

    Eq, EqEq, Bang, BangEq, Greater, GreaterEq, Lesser, LesserEq,
    And, Or, Print, Compt, If, Else, While, For, 
    Identifier(&'a [u8]), Literal(Literal)
}

#[derive(Debug, Copy, Clone)]
pub struct FullToken<'a>
{
    pub categ: Token<'a>,
    pub line: usize
}

#[derive(Debug, Copy, Clone)]
pub enum LexerError<'a>
{
    InvalidToken {token: &'a [u8], line: usize}
}

pub struct Lexer<'a>
{
    src: &'a [u8],
    line: usize,
    idx: usize,
    stops: HashSet<u8>
}

impl<'a> Lexer<'a>
{
    pub fn new(src: &'a str) -> Self
    {
        let mut ret = Self {src: src.as_bytes(), line: 1, idx: 0, stops: HashSet::new()};
        ret.stops.insert(b'[');
        ret.stops.insert(b']');
        ret.stops.insert(b'{');
        ret.stops.insert(b'}');
        ret.stops.insert(b'(');
        ret.stops.insert(b')');
        ret.stops.insert(b';');
        ret.stops.insert(b',');
        ret.stops.insert(b'.');
        ret.stops.insert(b'+');
        ret.stops.insert(b'-');
        ret.stops.insert(b'/');
        ret.stops.insert(b'*');
        ret.stops.insert(b' ');
        ret.stops.insert(b'\t');
        ret.stops.insert(b'\n');
        ret.stops.insert(b'\r');

        ret
    }
}

impl<'a> Iterator for Lexer<'a>
{
    type Item = Result<FullToken<'a>, LexerError<'a>>;

    fn next(&mut self) -> Option<Self::Item> 
    {
        loop
        {
            loop
            {
                if self.idx >= self.src.len()
                {
                    return None;
                }

                match self.src[self.idx]
                {
                    b' ' | b'\t' | b'\r' => self.idx += 1,
                    b'\n' => {self.line += 1; self.idx += 1;},
                    _ => break
                };
            }

            if self.idx + 1 < self.src.len() && self.src[self.idx] == b'/' && self.src[self.idx + 1] == b'/'
            {
                while self.idx < self.src.len() && self.src[self.idx] != b'\n' {self.idx += 1;}
                self.line += 1;
                self.idx += 1;
            }
            else
            {
                break;
            }
        }

        self.idx += 1;

        match self.src[self.idx - 1]
        {
            b'[' => return Some(Ok(FullToken {categ: Token::LeftSqr, line: self.line})),
            b']' => return Some(Ok(FullToken {categ: Token::RightSqr, line: self.line})),
            b'{' => return Some(Ok(FullToken {categ: Token::LeftBrace, line: self.line})),
            b'}' => return Some(Ok(FullToken {categ: Token::RightBrace, line: self.line})),
            b'(' => return Some(Ok(FullToken {categ: Token::LeftParen, line: self.line})),
            b')' => return Some(Ok(FullToken {categ: Token::RightParen, line: self.line})),
            b';' => return Some(Ok(FullToken {categ: Token::Semicolon, line: self.line})),
            b':' => return Some(Ok(FullToken {categ: Token::Colon, line: self.line})),
            b',' => return Some(Ok(FullToken {categ: Token::Comma, line: self.line})),
            b'.' => return Some(Ok(FullToken {categ: Token::Dot, line: self.line})),
            b'+' => return Some(Ok(FullToken {categ: Token::Plus, line: self.line})),
            b'-' => return Some(Ok(FullToken {categ: Token::Minus, line: self.line})),
            b'/' => return Some(Ok(FullToken {categ: Token::ForSlash, line: self.line})),
            b'*' => return Some(Ok(FullToken {categ: Token::Star, line: self.line})),
            b'?' => return Some(Ok(FullToken {categ: Token::Question, line: self.line})),
            _ => ()
        };

        let start = self.idx - 1;

        while self.idx < self.src.len() && !self.stops.contains(&self.src[self.idx])
        {
            self.idx += 1;   
        }

        match &self.src[start..self.idx]
        {
            [b'='] => Some(Ok(FullToken {categ: Token::Eq, line: self.line})),
            [b'=', b'='] => Some(Ok(FullToken {categ: Token::EqEq, line: self.line})),
            [b'!'] => Some(Ok(FullToken {categ: Token::Bang, line: self.line})),
            [b'!', b'='] => Some(Ok(FullToken {categ: Token::BangEq, line: self.line})),
            [b'>'] => Some(Ok(FullToken {categ: Token::Greater, line: self.line})),
            [b'>', b'='] => Some(Ok(FullToken {categ: Token::GreaterEq, line: self.line})),
            [b'<'] => Some(Ok(FullToken {categ: Token::Lesser, line: self.line})),
            [b'<', b'='] => Some(Ok(FullToken {categ: Token::LesserEq, line: self.line})),
            [b'&', b'&'] => Some(Ok(FullToken {categ: Token::And, line: self.line})),
            [b'|', b'|'] => Some(Ok(FullToken {categ: Token::Or, line: self.line})),
            [b'p', b'r', b'i', b'n', b't'] => Some(Ok(FullToken {categ: Token::Print, line: self.line})),
            [b'c', b'o', b'm', b'p', b't'] => Some(Ok(FullToken {categ: Token::Compt, line: self.line})),
            [b'i', b'f'] => Some(Ok(FullToken {categ: Token::If, line: self.line})),
            [b'e', b'l', b's', b'e'] => Some(Ok(FullToken {categ: Token::Else, line: self.line})),
            [b'w', b'h', b'i', b'l', b'e'] => Some(Ok(FullToken {categ: Token::While, line: self.line})),
            [b'f', b'o', b'r'] => Some(Ok(FullToken {categ: Token::For, line: self.line})),
            [b't', b'r', b'u', b'e'] => Some(Ok(FullToken {categ: Token::Literal(Literal::Bool(true)), line: self.line})),
            [b'f', b'a', b'l', b's', b'e'] => Some(Ok(FullToken {categ: Token::Literal(Literal::Bool(false)), line: self.line})),
            other => 
            {
                if other[0] == b'_' || other[0] >= b'a' && other[0] <= b'z' || other[0] >= b'A' && other[0] <= b'Z'
                {
                    Some(Ok(FullToken {categ: Token::Identifier(other), line: self.line}))
                }
                else if let Ok(i) = str::from_utf8(other).unwrap().parse::<i64>()
                {
                    Some(Ok(FullToken {categ: Token::Literal(Literal::Int(i)), line: self.line}))
                }
                else if let Ok(f) = str::from_utf8(other).unwrap().parse::<f64>()
                {
                    Some(Ok(FullToken {categ: Token::Literal(Literal::Float(f)), line: self.line}))
                }
                else
                {
                    Some(Err(LexerError::InvalidToken {token: other, line: self.line}))
                }
            }
        }
    }
}