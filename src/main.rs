use std::fs;
use std::env;
use logos::Logos;

mod lexer;
mod parser;
mod displays;
mod resolver;

use lexer::*;
use parser::*;
use resolver::*;

fn main()
{
    let args: Vec<String> = env::args().collect();
    if args.len() != 2
    {
        println!("Usage: [binary] [path/to/file]");
        return;
    }

    let content = fs::read_to_string(&args[1]).expect("Could not read file.");

    struct TokenError<'a>
    {
        err: LexerError,
        str: &'a [u8]
    }

    let mut lex_errs = Vec::<TokenError>::new();
    let tokens: Vec<Token> = Token::lexer(&content).spanned().filter_map(|(rt, span)| 
        match rt 
        {
            Ok(t) => Some(t), 
            Err(err) => 
            {
                lex_errs.push(TokenError {err, str: &content[span.start..span.end].as_bytes()}); 
                None
            }
        }
    ).collect();

    if lex_errs.len() > 0
    {
        for err in lex_errs {println!("Invalid token {}", str::from_utf8(err.str).unwrap_or("<invalid_utf8"))};
        return;
    }

    let mut parser = Parser::new();
    parser.parse(&mut tokens.iter().peekable());
    if parser.errors.len() > 0
    {
        for err in parser.errors {println!("Parse error: {}", err);}
        println!("Panic mode AST: {}", parser.ast);
        return;
    }

    println!("Successfully parsed. AST: {}", parser.ast);

    let mut resolver = Resolver::new();
    resolver.check(&parser.ast);
    if resolver.errors.len() > 0
    {
        for err in resolver.errors {println!("Resolution error: {}", err);}
        println!("Panic mode AST: {}", resolver.ast);
        return;
    }

    println!("Successfully resolved. AST: {}", resolver.ast);

    return;
}
