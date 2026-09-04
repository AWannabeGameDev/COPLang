use std::env;
use std::fs;
use std::process;
use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;
lalrpop_mod!(parser);
mod resolver;
mod displays;

use lexer::*;
use parser::*;

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
    let source_code = fs::read_to_string(file_path).unwrap_or_else(
        |_| 
        {
            eprintln!("Fatal: Could not read file at {}", file_path);
            process::exit(1);
        }
    );

    let lexer = LexerWrapper::new(&source_code);
    let parser = PROGParser::new();
    match parser.parse(lexer) 
    {
        Ok(parsed_ast) => 
        {
            println!("Successfully parsed the source file.");
            println!("--- AST ---");
            println!("{}", parsed_ast); 

            // Initialize the resolver and pass the AST[cite: 2]
            let mut resolver = resolver::Resolver::new();
            let (res_ast, errors) = resolver.resolve(parsed_ast);

            // Dump errors if the resolver found type mismatches, missing idens, etc.[cite: 2]
            if !errors.is_empty()
            {
                eprintln!("Resolution Errors:");
                for err in errors
                {
                    eprintln!("{}", err);
                }
                process::exit(1);
            }

            println!("--- Resolved AST ---");
            println!("{}", res_ast);
        }
        Err(e) => 
        {
            eprintln!("Parse Error: {:?}", e);
            process::exit(1);
        }
    }
}