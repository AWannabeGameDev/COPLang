use std::env;
use std::fs;
use std::process;
use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;
lalrpop_mod!(parser);
mod displays;

use lexer::*;
use parser::*;

fn main() 
{
    // 1. Grab the file path from CLI arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 
    {
        eprintln!("Usage: cargo run -- <file_path>");
        process::exit(1);
    }
    
    let file_path = &args[1];

    // 2. Read the source file into a String
    let source_code = fs::read_to_string(file_path).unwrap_or_else(
        |_| 
        {
            eprintln!("Fatal: Could not read file at {}", file_path);
            process::exit(1);
        }
    );

    // 3. Initialize your Logos lexer wrapper[cite: 3]
    let lexer = LexerWrapper::new(&source_code);

    // 4. Fire up the LALRPOP parser
    // Because your top-level public rule is `ASSN`, 
    // LALRPOP specifically generates a struct named `ASSNParser`.
    let parser = PROGParser::new();

    // 5. Parse the token stream
    match parser.parse(lexer) 
    {
        Ok(parsed_ast) => 
        {
            println!("Successfully parsed the source file.");
            
            // Uncomment this once you add #[derive(Debug)] to your AST nodes!
            println!("{}", parsed_ast); 
        }
        Err(e) => 
        {
            eprintln!("Parse Error: {:?}", e);
            process::exit(1);
        }
    }
}