use std::env::args;
use std::error::Error;
use std::fs::read;

mod lexer;
use lexer::Lexer;

fn main() -> Result<(), Box<dyn Error>> {
    let contents;
    if let Some(file) = args().nth(1) {
        contents = read(file)?;
    } else {
        panic!("No files given");
    }
    let mut lexer = Lexer::new(&contents);
    lexer.lex().unwrap();
    let tokens = lexer.get_tokens();
    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}
