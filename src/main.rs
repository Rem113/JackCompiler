#[macro_use]
extern crate lazy_static;

use std::collections::VecDeque;
use std::env::args;
use std::fs::{read_dir, read_to_string, File};
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

mod tokenizer;
use tokenizer::{Token, TokenType, Tokenizer};

mod parser;
use parser::Parser;

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

fn vm_from_tokens(tokens: Vec<Token>) -> String {
    let mut parser = Parser::new(VecDeque::from(tokens));

    parser.parse()
}

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        panic!("Usage: jack <path>");
    };

    let dir_path = Path::new(&args[1]);

    let read_dir = match read_dir(dir_path) {
        Ok(dir) => dir,
        Err(_) => panic!("Invalid path"),
    };

    read_dir
        .filter_map(|file| match file {
            Ok(file) => {
                let file_name = file.file_name().to_string_lossy().into_owned();
                if file_name.ends_with(".jack") {
                    Some(file_name)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .for_each(|file| {
            let path_string = &format!("{}/{}", args[1], file);
            let path = Path::new(path_string);
            let tokens = tokens_for_file(path);
            let vm = vm_from_tokens(tokens);

            let out_path_string = &format!("{}/{}.vm", args[1], file);
            let out_path = Path::new(out_path_string);
            let file = match File::create(out_path) {
                Ok(file) => file,
                Err(err) => panic!("{}", err),
            };
            let mut writer = BufWriter::new(file);
            match writer.write(vm.as_bytes()) {
                Ok(_) => {}
                Err(err) => panic!("{}", err),
            };
        });
}
