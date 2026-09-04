use std::env;
use std::fs;
use std::process;

use lalrpop_util::lalrpop_mod;
use lalrpop_util::ParseError;

mod lexer;
mod ast;
lalrpop_mod!(parser);
mod resolver;
mod displays;

use lexer::*;
use parser::*;
use resolver::*;

fn main() 
{
    // 1. Grab the file path from CLI arguments[cite: 3]
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 
    {
        eprintln!("Usage: cargo run -- <file_path>");
        process::exit(1);
    }
    
    let file_path = &args[1];
    let src = fs::read_to_string(file_path).unwrap_or_else(
        |_| 
        {
            eprintln!("Fatal: Could not read file at {}", file_path);
            process::exit(1);
        }
    );

    let lexer = Lexer::new(&src);
    let mut lex_errs = Vec::<LexError>::new();
    let tokens: Vec<_> = lexer.map(
        |it| match it 
        {
            Ok(token) => token, 
            Err(err) => 
            {
                lex_errs.push(err);
                match err
                {
                    LexError::InvalidToken(range) => (Token::Error, range),
                }
            }
        }
    ).collect();

    if lex_errs.len() > 0
    {
        for err in lex_errs 
        {
            print!("Lex error: ");
            match err
            {
                LexError::InvalidToken(span) => println!("Invalid token {} at span {}:{}", 
                    &src[span.start..span.end], span.start, span.end)
            }
        }
    }

    let parser = PROGParser::new();
    let mut parse_errs: Vec<ParseError<usize, Token, ()>> = Vec::new();
    let ast = parser.parse(&mut parse_errs, tokens.into_iter().map(|(token, span)| Ok((span.start, token, span.end)))).unwrap();

    if parse_errs.len() > 0
    {
        for err in parse_errs 
        {
            print!("Parse error: ");
            match err
            {
                ParseError::InvalidToken {location} => println!("Invalid token at byte {location}."),
                ParseError::UnrecognizedEof {expected, ..} => 
                {
                    print!("Expected ");
                    for exp in expected {print!("{exp}, ")}
                    println!("found EOF.");
                },
                ParseError::UnrecognizedToken {token: (l, tok, r), expected} =>
                {
                    print!("Expected ");
                    for exp in expected {print!("{exp}, ")}
                    println!("found {tok} at span {l}:{r}.");
                },
                ParseError::ExtraToken {token: (l, tok, r)} =>
                {
                    print!("Expected EOF, found {tok} at span {l}:{r}")
                },
                ParseError::User {..} => unreachable!(),
            }
        }
    }
    else
    {
        println!("Successfully parsed:\n{ast}")
    }

    let mut resolver = Resolver::new();
    let (res_ast, res_errors) = resolver.resolve(&ast);

    if res_errors.len() > 0
    {
        for err in res_errors {println!("Resolution error: {err}")}
    }
    else
    {
        println!("Successfully resolved:\n{res_ast}")
    }
}