use regex::{Match, Regex};
use strum_macros::Display;

#[derive(Display, PartialEq, Clone)]
pub enum TokenType {
	Keyword,
	Symbol,
	IntegerConstant,
	StringConstant,
	Identifier,
	EndOfFile,
}

#[derive(Clone)]
pub struct Token {
	pub token: TokenType,
	pub value: String,
}

pub struct Tokenizer {
	code: String,
}

lazy_static! {
	static ref KEYWORDS: Regex = Regex::new("^(class|constructor|function|method|field|static|var|int|char|boolean|void|true|false|null|this|let|do|if|else|while|return)").unwrap();
	static ref SYMBOLS: Regex = Regex::new(r#"^[\{\}\(\)\[\].,;\+\-\*/&|<>=~]"#).unwrap();
	static ref IDENTIFIERS: Regex = Regex::new("^[_[:alpha:]][_[:alnum:]]*").unwrap();
	static ref INTEGER_CONSTANTS: Regex = Regex::new("^[0-9]{1,5}").unwrap();
	static ref STRING_CONSTANTS: Regex = Regex::new(r#"^".*""#).unwrap();
	static ref COMMENTS: Regex = Regex::new("^//.*").unwrap();
}

impl Tokenizer {
	fn remove_n_first_chars(&mut self, count: usize) {
		for _ in 0..count {
			self.code.remove(0);
		}
	}

	pub fn new(code: String) -> Tokenizer {
		Tokenizer { code: code }
	}

	// Returns the next token in the code
	pub fn next(&mut self) -> Token {
		self.code = self.code.trim_start().to_owned();
		let code = &self.code;

		if COMMENTS.is_match(&code) {
			let bounds: Match = COMMENTS.find(&code).unwrap();

			self.remove_n_first_chars(bounds.end() - bounds.start());

			return self.next();
		};

		if KEYWORDS.is_match(&code) {
			let bounds: Match = KEYWORDS.find(&code).unwrap();
			let value: String = code[bounds.start()..bounds.end()].to_owned();

			self.remove_n_first_chars(value.len());

			return Token {
				token: TokenType::Keyword,
				value: value,
			};
		};

		if SYMBOLS.is_match(&code) {
			let bounds: Match = SYMBOLS.find(&code).unwrap();
			let value: String = code[bounds.start()..bounds.end()].to_owned();

			self.remove_n_first_chars(value.len());

			return Token {
				token: TokenType::Symbol,
				value: value,
			};
		};

		if INTEGER_CONSTANTS.is_match(&code) {
			let bounds: Match = INTEGER_CONSTANTS.find(&code).unwrap();
			let value: String = code[bounds.start()..bounds.end()].to_owned();

			self.remove_n_first_chars(value.len());

			return Token {
				token: TokenType::IntegerConstant,
				value: value,
			};
		};

		if STRING_CONSTANTS.is_match(&code) {
			let bounds: Match = STRING_CONSTANTS.find(&code).unwrap();
			let value: String = code[bounds.start()..bounds.end()].to_owned();

			self.remove_n_first_chars(value.len());

			return Token {
				token: TokenType::StringConstant,
				value: value,
			};
		};

		if IDENTIFIERS.is_match(&code) {
			let bounds: Match = IDENTIFIERS.find(&code).unwrap();
			let value: String = code[bounds.start()..bounds.end()].to_owned();

			self.remove_n_first_chars(value.len());

			return Token {
				token: TokenType::Identifier,
				value: value,
			};
		};

		return Token {
			token: TokenType::EndOfFile,
			value: String::new(),
		};
	}
}
