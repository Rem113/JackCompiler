#[macro_use]
extern crate lazy_static;

use std::collections::VecDeque;
use std::fs::read_to_string;
use std::path::Path;

mod tokenizer;
use tokenizer::{Token, TokenType, Tokenizer};

mod parser;
use parser::Parser;

mod tree;
use tree::Tree;

fn tokens_for_file(path: &Path) -> Vec<Token> {
    let content = match read_to_string(path) {
        Ok(content) => content,
        Err(err) => panic!("{}", err),
    };

    let mut tokenizer = Tokenizer::new(content);

    let mut result = Vec::new();

    loop {
        let token = tokenizer.next();
        if token.token == TokenType::EndOfFile {
            return result;
        }
        result.push(token);
    }
}

fn tree_from_tokens(tokens: Vec<Token>) -> Tree {
    let mut parser = Parser::new(VecDeque::from(tokens));

    parser.parse()
}

fn main() {
    let path = Path::new("src/Main.jack");

    let tokens = tokens_for_file(path);

    // tokens.iter().for_each(|token| println!("{}", token.value));

    let tree = tree_from_tokens(tokens);

    println!("{}", tree.to_xml());
}
