use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[token("let")]
    Let,

    #[token("fn")]
    Fn,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex("[0-9]+")]
    Integer,

    #[token("=")]
    Assign,

    #[token("+")]
    Plus,

    #[regex(r"[ ]+", logos::skip)]
    Whitespace,
}

pub fn compile_source(source: &str) {
    let mut lexer = Token::lexer(source);
    println!("--- Lexing Results ---");
    while let Some(token_result) = lexer.next() {
        match token_result {
            Ok(token) => println!("Token: {:?}", token),
            Err(_) => println!("Error: Invalid token at {:?}", lexer.span()),
        }
    }
    println!("--- Emitting LLVM IR / Assembly (AOT Placeholder) ---");
    println!("; Generated AOT Code Template");
}
