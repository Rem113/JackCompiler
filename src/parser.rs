use crate::tokenizer::{Token, TokenType};

use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
struct Symbol {
	pub kind: String,
	pub typing: String,
	pub index: u8,
}

pub struct Parser {
	tokens: VecDeque<Token>,
	class_name: String,
	class_symbol_table: HashMap<String, Symbol>,
	func_symbol_table: HashMap<String, Symbol>,
}

impl Parser {
	fn find_symbol(&self, map: &HashMap<String, Symbol>, name: &String) -> Symbol {
		map.get(name).unwrap().clone()
	}

	fn find_symbol_in_class(&self, name: &String) -> Symbol {
		self.find_symbol(&self.class_symbol_table, name)
	}

	fn new_func_symbol_table(&mut self) {
		self.func_symbol_table = HashMap::new();
	}

	fn find_symbol_in_func(&self, name: &String) -> Symbol {
		self.find_symbol(&self.func_symbol_table, name)
	}

	fn add_symbol_in_class(&mut self, name: &String, kind: &String, typing: &String) {
		let mut index = 0;
		let same_kind: Vec<(&String, &Symbol)> = self
			.class_symbol_table
			.iter()
			.filter(|(_, sym)| sym.kind == *kind)
			.collect();
		if same_kind.len() != 0 {
			let (_, max_sym) = same_kind
				.iter()
				.max_by(|(_, sym1), (_, sym2)| sym1.index.cmp(&sym2.index))
				.unwrap();
			index = max_sym.index + 1;
		};
		self.class_symbol_table.insert(
			name.to_string(),
			Symbol {
				kind: kind.to_string(),
				typing: typing.to_string(),
				index: index,
			},
		);
	}

	fn add_symbol_in_func(&mut self, name: &String, kind: &String, typing: &String) {
		let mut index = 0;
		let same_kind: Vec<(&String, &Symbol)> = self
			.func_symbol_table
			.iter()
			.filter(|(_, sym)| sym.kind == *kind)
			.collect();
		if same_kind.len() != 0 {
			let (_, max_sym) = same_kind
				.iter()
				.max_by(|(_, sym1), (_, sym2)| sym1.index.cmp(&sym2.index))
				.unwrap();
			index = max_sym.index + 1;
		};
		self.func_symbol_table.insert(
			name.to_string(),
			Symbol {
				kind: kind.to_string(),
				typing: typing.to_string(),
				index: index,
			},
		);
	}

	fn next(&mut self) -> Token {
		self.tokens.pop_front().unwrap()
	}

	fn peek(&mut self) -> Token {
		self.tokens.front().unwrap().clone()
	}

	fn parse_class(&mut self) -> String {
		self.next(); // class
		self.class_name = self.next().value;
		self.next(); // {

		// Optional class variables declaration
		loop {
			let next_token = self.peek();

			if next_token.value != "static" && next_token.value != "field" {
				break;
			};

			self.parse_class_var_dec();
		}

		let mut result = String::new();

		// Optional subroutines declaration
		loop {
			let next_token = self.peek();

			if next_token.value != "constructor"
				&& next_token.value != "function"
				&& next_token.value != "method"
			{
				break;
			};

			result.push_str(&self.parse_subroutine_dec());
		}

		self.next(); // }
		result
	}

	fn parse_subroutine_dec(&mut self) -> String {
		self.new_func_symbol_table();
		let mut result = String::new();
		let kind = self.next().value; // function, method or constructor

		if kind != "method" {
			self.add_symbol_in_func(
				&"this".to_string(),
				&"argument".to_string(),
				&self.class_name.clone(),
			)
		}

		self.next(); // void or type

		let subroutine_name = self.parse_subroutine_name();
		self.next().value; // {

		self.parse_parameter_list();

		self.next(); // {
		result.push_str(&self.parse_subroutine_body());
		result.push_str("</subroutine_dec>\n");
		result
	}

	fn parse_subroutine_name(&mut self) -> String {
		self.next().value
	}

	fn parse_subroutine_body(&mut self) -> String {
		let mut result = String::new();
		self.next(); // {

		loop {
			let var_or_else = self.peek();

			if var_or_else.value != "var" {
				break;
			}

			self.parse_var_dec();
		}

		result.push_str(&self.parse_statements());

		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str("</subroutine_body>\n");
		result
	}

	fn parse_var_dec(&mut self) {
		self.next(); // var
		let typing = self.parse_type();
		let mut name = self.parse_var_name();

		loop {
			self.add_symbol_in_func(&name, &"local".to_string(), &typing);

			let next_token = self.peek();

			if next_token.value == ";" {
				self.next(); // ;
			}

			self.next(); // ,
			name = self.parse_var_name();
		}
	}

	fn parse_statements(&mut self) -> String {
		let mut result = String::new();

		loop {
			let next_elem = self.peek().value;

			if next_elem != "let"
				&& next_elem != "if"
				&& next_elem != "while"
				&& next_elem != "do"
				&& next_elem != "return"
			{
				return result;
			}

			result.push_str(&self.parse_statement());
		}
	}

	fn parse_statement(&mut self) -> String {
		match self.peek().value.as_str() {
			"let" => self.parse_let_statement(),
			"if" => self.parse_if_statement(),
			"while" => self.parse_while_statement(),
			"do" => self.parse_do_statement(),
			"return" => self.parse_return_statement(),
			_ => panic!("An error has occured"),
		}
	}

	fn parse_let_statement(&mut self) -> String {
		let mut result = String::new();
		self.next(); // let
		let var_name = self.parse_var_name();

		if self.peek().value == "[" {
			self.next(); // [
			result.push_str(&self.parse_expression());
			self.next(); // ]
		}

		self.next(); // =
		result.push_str(&self.parse_expression());
		self.next(); // ;
		result
	}

	fn parse_if_statement(&mut self) -> String {
		let mut result = String::new();
		self.next(); // if
		self.next(); // (

		result.push_str(&self.parse_expression());

		self.next(); // )
		self.next(); // {

		result.push_str(&self.parse_statements());
		self.next(); // }
		if self.peek().value == "else" {
			self.next(); // else
			self.next(); // {

			result.push_str(&self.parse_statements());

			self.next(); // }
		}

		result
	}

	fn parse_while_statement(&mut self) -> String {
		let mut result = String::new();
		self.next(); // while
		self.next(); // (
		result.push_str(&self.parse_expression());
		self.next(); // )
		self.next(); // {
		result.push_str(&self.parse_statements());
		self.next(); // }
		result
	}

	fn parse_do_statement(&mut self) -> String {
		let mut result = String::new();
		self.next(); // do
		result.push_str(&self.parse_subroutine_call());
		self.next(); // ;
		result
	}

	fn parse_return_statement(&mut self) -> String {
		let mut result = String::new();
		self.next(); // return
		if self.peek().value != ";" {
			result.push_str(&self.parse_expression());
		};
		self.next(); // ;
		result
	}

	fn parse_subroutine_call(&mut self) -> String {
		let mut result = String::new();
		let func_or_class = self.next();

		match self.peek().value.as_str() {
			"." => {
				self.tokens.insert(0, func_or_class);
				result.push_str(&self.parse_class_name());
				self.next(); // .
				result.push_str(&self.parse_subroutine_name());
			}
			_ => {
				result.push_str(&self.parse_subroutine_name());
			}
		};

		self.next(); // (

		if self.peek().value != ")" {
			result.push_str(&self.parse_expression_list());
		};

		self.next(); // )

		result
	}

	fn parse_expression_list(&mut self) -> String {
		let mut result = String::new();

		loop {
			result.push_str(&self.parse_expression());

			if self.peek().value != "," {
				return result;
			};

			self.next(); // ,
		}
	}

	fn parse_expression(&mut self) -> String {
		let mut result = String::new();
		result.push_str(&self.parse_term());

		loop {
			let op_or_else = self.peek();

			if op_or_else.value != "+"
				&& op_or_else.value != "-"
				&& op_or_else.value != "*"
				&& op_or_else.value != "/"
				&& op_or_else.value != "&"
				&& op_or_else.value != "|"
				&& op_or_else.value != "<"
				&& op_or_else.value != ">"
				&& op_or_else.value != "="
			{
				return result;
			}

			result.push_str(&self.parse_op());
			result.push_str(&self.parse_term());
		}
	}

	fn parse_op(&mut self) -> String {
		self.next().value
	}

	fn parse_term(&mut self) -> String {
		let mut result = String::new();

		let next_token = self.peek();

		if next_token.token == TokenType::IntegerConstant {
			result.push_str(&self.parse_integer_constant());
			return result;
		};
		if next_token.token == TokenType::StringConstant {
			result.push_str(&self.parse_string_constant());
			return result;
		};
		if next_token.value == "true"
			|| next_token.value == "false"
			|| next_token.value == "null"
			|| next_token.value == "this"
		{
			result.push_str(&self.parse_keyword_constant());
			return result;
		};

		if next_token.value == "(" {
			self.next(); // (
			result.push_str(&self.parse_expression());
			self.next(); // )
			return result;
		};

		if next_token.value == "-" || next_token.value == "~" {
			result.push_str(&self.parse_unary_op());
			result.push_str(&self.parse_term());
			return result;
		};

		// Var name or subroutine call
		let var_name_or_sub_name = self.next();

		let bracket_or_else = self.peek();

		// Var[]
		if bracket_or_else.value == "[" {
			let var_name = var_name_or_sub_name.value;
			self.next(); // [
			result.push_str(&self.parse_expression());
			self.next(); // ]
			return result;
		};

		// Subroutine
		if bracket_or_else.value == "(" || bracket_or_else.value == "." {
			self.tokens.insert(0, var_name_or_sub_name);
			result.push_str(&self.parse_subroutine_call());
			return result;
		};

		// Var name
		self.tokens.insert(0, var_name_or_sub_name);

		result.push_str(&self.parse_var_name());
		return result;
	}

	fn parse_unary_op(&mut self) -> String {
		self.next().value
	}

	fn parse_integer_constant(&mut self) -> String {
		self.next().value
	}

	fn parse_string_constant(&mut self) -> String {
		self.next().value
	}

	fn parse_keyword_constant(&mut self) -> String {
		self.next().value
	}

	fn parse_parameter_list(&mut self) {
		let type_or_else = self.peek();

		if type_or_else.value != "int"
			&& type_or_else.value != "char"
			&& type_or_else.value != "boolean"
			&& type_or_else.token != TokenType::Identifier
		{
			return;
		};

		loop {
			let typing = self.next().value;
			let name = self.parse_var_name();

			self.add_symbol_in_func(&name, &"argument".to_string(), &typing);

			let comma_or_else = self.next();

			if comma_or_else.value != "," {
				return;
			};
		}
	}

	fn parse_class_var_dec(&mut self) {
		let kind = self.next().value; // static or field
		let typing = self.parse_type(); // int, char, boolean or class name

		loop {
			let name = self.parse_var_name();

			self.add_symbol_in_class(&name, &kind, &typing);

			// Check for other variable declarations
			let comma_or_semi = self.next();

			if comma_or_semi.value == ";" {
				return;
			}
		}
	}

	fn parse_var_name(&mut self) -> String {
		self.next().value
	}

	fn parse_type(&mut self) -> String {
		let front_token = self.peek();

		if front_token.value == "int" || front_token.value == "char" || front_token.value == "boolean" {
			self.next().value
		} else {
			self.parse_class_name()
		}
	}

	fn parse_class_name(&mut self) -> String {
		self.next().value
	}

	pub fn new(tokens: VecDeque<Token>) -> Parser {
		Parser {
			tokens: tokens,
			class_name: String::new(),
			class_symbol_table: HashMap::new(),
			func_symbol_table: HashMap::new(),
		}
	}

	pub fn parse(&mut self) -> String {
		self.parse_class()
	}
}
