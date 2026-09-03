use logos::{Logos, SpannedIter};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Literal {
    Int(i64), 
    Float(f64), 
    Bool(bool)
}

#[derive(Logos, PartialEq, Debug, Copy, Clone)]
#[logos(error = LexerError)]
#[logos(skip(r"[ \t\n\f]+|//[^\n]*", allow_greedy = true))]
pub enum Token<'a> 
{
    // Punctuation
    #[token("[")] LeftSqr, 
    #[token("]")] RightSqr, 
    #[token("{")] LeftBrace, 
    #[token("}")] RightBrace, 
    #[token("(")] LeftParen, 
    #[token(")")] RightParen, 
    #[token(";")] Semicolon, 
    #[token(":")] Colon, 
    #[token(",")] Comma, 
    #[token(".")] Dot, 
    #[token("?")] Question, 
    
    // Math Ops
    #[token("+")] Plus, 
    #[token("-")] Minus, 
    #[token("/")] ForSlash, 
    #[token("*")] Star,

    // Logic & Comparison
    #[token("=")] Eq, 
    #[token("==")] EqEq, 
    #[token("!")] Bang, 
    #[token("!=")] BangEq, 
    #[token(">")] Greater, 
    #[token(">=")] GreaterEq, 
    #[token("<")] Lesser, 
    #[token("<=")] LesserEq,
    #[token("&&")] And, 
    #[token("||")] Or,

    // Keywords 
    #[token("Int")] Int,
    #[token("Float")] Float,
    #[token("Bool")] Bool,
    #[token("print")] Print, 
    #[token("compt")] Compt, 
    #[token("if")] If, 
    #[token("else")] Else, 
    #[token("while")] While, 
    #[token("for")] For,
    #[token("entt")] Entt,
    
    // Identifiers (borrowed directly from source as a byte slice)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().as_bytes())]
    Identifier(&'a [u8]), 
    
    // Literals (Stacking macros to route them all into the same variant)
    #[regex(r"[0-9]+", |lex| Literal::Int(lex.slice().parse().unwrap()))]
    #[regex(r"[0-9]+\.[0-9]+", |lex| Literal::Float(lex.slice().parse().unwrap()))]
    #[token("true", |_| Literal::Bool(true))]
    #[token("false", |_| Literal::Bool(false))]
    Literal(Literal)
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum LexerError
{
    #[default] InvalidToken
}

pub struct LexerWrapper<'a> 
{
    inner: SpannedIter<'a, Token<'a>>,
}

impl<'a> LexerWrapper<'a> 
{
    pub fn new(input: &'a str) -> Self 
    {
        Self {inner: Token::lexer(input).spanned()}
    }
}

impl<'a> Iterator for LexerWrapper<'a> 
{
    type Item = Result<(usize, Token<'a>, usize), LexerError>;

    fn next(&mut self) -> Option<Self::Item> 
    {
        self.inner.next().map(|(token_res, span)| 
        {
            match token_res 
            {
                Ok(token) => Ok((span.start, token, span.end)),
                Err(e) => Err(e),
            }
        })
    }
}