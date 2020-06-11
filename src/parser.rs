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
	label_count: u8,
}

impl Parser {
	fn get_func_local_count(&self) -> usize {
		self
			.func_symbol_table
			.iter()
			.filter(|(_, symbol)| symbol.kind == "local")
			.count()
	}

	fn get_class_field_count(&self) -> usize {
		self
			.class_symbol_table
			.iter()
			.filter(|(_, symbol)| symbol.kind == "field")
			.count()
	}

	fn get_label(&mut self) -> String {
		self.label_count += 1;
		String::from(&format!("{}{}", self.class_name, self.label_count - 1))
	}

	fn find_symbol(&self, name: &String) -> Option<&Symbol> {
		match self.func_symbol_table.get(name) {
			Some(sym) => Some(sym),
			None => self.class_symbol_table.get(name),
		}
	}

	fn new_func_symbol_table(&mut self) {
		self.func_symbol_table = HashMap::new();
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

		if kind == "method" {
			self.add_symbol_in_func(
				&"this".to_string(),
				&"argument".to_string(),
				&self.class_name.clone(),
			);
		}

		self.next(); // void or type

		let subroutine_name = self.parse_subroutine_name();
		self.next().value; // {

		self.parse_parameter_list();

		self.next(); // {

		let subroutine_body = self.parse_subroutine_body();
		let local_count = self.get_func_local_count();
		result.push_str(&format!(
			"function {}.{} {}\n",
			self.class_name, subroutine_name, local_count
		));
		if kind == "constructor" {
			result.push_str(&format!("push constant {}\n", self.get_class_field_count()));
			result.push_str("call Memory.alloc 1\n");
			result.push_str("pop pointer 0\n");
		}
		if kind == "method" {
			result.push_str("push argument 0\n");
			result.push_str("pop pointer 0\n");
		}
		result.push_str(&subroutine_body);

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

		self.next(); // }
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
				return;
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

		let mut offset_code = String::new();

		if self.peek().value == "[" {
			self.next(); // [
			offset_code.push_str(&self.parse_expression());
			self.next(); // ]
		}

		self.next(); // =
		result.push_str(&self.parse_expression());
		self.next(); // ;

		// Symbol of the assigned variable
		let symbol = match self.find_symbol(&var_name) {
			Some(sym) => sym,
			None => panic!("An error has occured"),
		};

		if offset_code.len() != 0 {
			if symbol.kind == "field" {
				result.push_str(&format!("push this {}\n", symbol.index));
			} else {
				result.push_str(&format!("push {} {}\n", symbol.kind, symbol.index));
			};
			result.push_str(offset_code.as_str());
			result.push_str("add\n");
			result.push_str("pop pointer 1\n");
			result.push_str("pop that 0\n");
		} else {
			if symbol.kind == "field" {
				result.push_str(&format!("pop this {}\n", symbol.index));
			} else {
				result.push_str(&format!("pop {} {}\n", symbol.kind, symbol.index));
			};
		}

		result
	}

	fn parse_if_statement(&mut self) -> String {
		let mut result = String::new();
		self.next(); // if
		self.next(); // (

		result.push_str(&self.parse_expression());
		result.push_str("not\n");
		let label_false = self.get_label();
		let label_true = self.get_label();
		result.push_str(&format!("if-goto {}\n", label_false));

		self.next(); // )
		self.next(); // {

		result.push_str(&self.parse_statements());
		result.push_str(&format!("goto {}\n", label_true));
		result.push_str(&format!("label {}\n", label_false));
		self.next(); // }
		if self.peek().value == "else" {
			self.next(); // else
			self.next(); // {

			result.push_str(&self.parse_statements());

			self.next(); // }
		}
		result.push_str(&format!("label {}\n", label_true));

		result
	}

	fn parse_while_statement(&mut self) -> String {
		let mut result = String::new();
		let loop_label = self.get_label();
		let end_label = self.get_label();
		result.push_str(&format!("label {}\n", loop_label));
		self.next(); // while
		self.next(); // (
		result.push_str(&self.parse_expression());
		result.push_str("not\n");
		result.push_str(&format!("if-goto {}\n", end_label));
		self.next(); // )
		self.next(); // {
		result.push_str(&self.parse_statements());
		result.push_str(&format!("goto {}\n", loop_label));
		result.push_str(&format!("label {}\n", end_label));
		self.next(); // }
		result
	}

	fn parse_do_statement(&mut self) -> String {
		let mut result = String::new();
		self.next(); // do
		result.push_str(&self.parse_subroutine_call());
		result.push_str("pop temp 0\n");
		self.next(); // ;
		result
	}

	fn parse_return_statement(&mut self) -> String {
		let mut result = String::new();
		self.next(); // return
		if self.peek().value != ";" {
			result.push_str(&self.parse_expression());
		} else {
			result.push_str("push constant 0\n");
		};
		result.push_str("return\n");
		self.next(); // ;
		result
	}

	fn parse_subroutine_call(&mut self) -> String {
		let mut result = String::new();
		let func_or_class_name = self.next();
		let mut function_name = String::new();
		let mut param_count = 0;

		match self.peek().value.as_str() {
			"." => {
				self.tokens.insert(0, func_or_class_name.clone());
				let class_or_instance_name = self.parse_class_name();
				self.next(); // .
				let subroutine_name = self.parse_subroutine_name();

				match self.find_symbol(&class_or_instance_name) {
					Some(symbol) => {
						match symbol.kind.as_str() {
							"field" => result.push_str(&format!("push this {}\n", symbol.index)),
							"argument" | "static" | "local" => {
								result.push_str(&format!("push {} {}\n", symbol.kind, symbol.index))
							}
							_ => panic!("An error has occured"),
						};
						function_name.push_str(&format!("{}.{}", symbol.typing, subroutine_name));
						param_count += 1;
					}
					None => {
						function_name.push_str(&format!("{}.{}", class_or_instance_name, subroutine_name))
					}
				};
			}
			_ => {
				param_count += 1;
				result.push_str("push pointer 0\n");
				function_name.push_str(&format!("{}.{}", self.class_name, func_or_class_name.value));
			}
		};

		self.next(); // (

		if self.peek().value != ")" {
			let (count, code) = self.parse_expression_list();
			param_count += count;
			result.push_str(&code);
		};

		self.next(); // )

		result.push_str(&format!("call {} {}\n", function_name, param_count));

		result
	}

	fn parse_expression_list(&mut self) -> (u8, String) {
		let mut result = String::new();

		let mut count = 1;

		loop {
			result.push_str(&self.parse_expression());

			if self.peek().value != "," {
				return (count, result);
			};

			count += 1;

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
				&& op_or_else.value != "~"
			{
				return result;
			}

			let op = self.parse_op();
			result.push_str(&self.parse_term());

			match op.as_str() {
				"+" => result.push_str("add\n"),
				"-" => result.push_str("sub\n"),
				"*" => result.push_str("call Math.multiply 2\n"),
				"/" => result.push_str("call Math.divide 2\n"),
				"&" => result.push_str("and\n"),
				"|" => result.push_str("or\n"),
				"<" => result.push_str("lt\n"),
				">" => result.push_str("gt\n"),
				"=" => result.push_str("eq\n"),
				"~" => result.push_str("neg\n"),
				_ => panic!("An error has occured"),
			}
		}
	}

	fn parse_op(&mut self) -> String {
		self.next().value
	}

	fn parse_term(&mut self) -> String {
		let mut result = String::new();

		let next_token = self.peek();

		if next_token.token == TokenType::IntegerConstant {
			let integer_constant = self.parse_integer_constant();
			result.push_str(&format!("push constant {}\n", integer_constant));
			return result;
		};
		if next_token.token == TokenType::StringConstant {
			let string_constant = self.parse_string_constant();
			result.push_str(&format!("push constant {}\n", string_constant.len()));
			result.push_str(&format!("call String.new 1\n"));

			for c in string_constant.chars() {
				result.push_str(&format!("push constant {}\n", c as u8));
				result.push_str(&format!("call String.appendChar 2\n"));
			}

			return result;
		};
		if next_token.value == "true"
			|| next_token.value == "false"
			|| next_token.value == "null"
			|| next_token.value == "this"
		{
			let keyword_constant = self.parse_keyword_constant();

			match keyword_constant.as_str() {
				"true" => {
					result.push_str("push constant 0\n");
					result.push_str("not\n")
				}
				"false" => result.push_str("push constant 0\n"),
				"null" => result.push_str("push constant 0\n"),
				"this" => result.push_str("push pointer 0\n"),
				_ => panic!("An error has occured"),
			}

			return result;
		};

		// (expression)
		if next_token.value == "(" {
			self.next(); // (
			result.push_str(&self.parse_expression());
			self.next(); // )
			return result;
		};

		// unary_op term
		if next_token.value == "-" || next_token.value == "~" {
			let unary_op = self.parse_unary_op();

			result.push_str(&self.parse_term());

			match &unary_op[..] {
				"-" => result.push_str("neg\n"),
				"~" => result.push_str("not\n"),
				_ => panic!("An error has occured"),
			};

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

			let symbol = match self.find_symbol(&var_name) {
				Some(sym) => sym,
				None => panic!("An error has occured"),
			};

			match symbol.kind.as_str() {
				"field" => result.push_str(&format!("push this {}\n", symbol.index)),
				"argument" | "static" | "local" => {
					result.push_str(&format!("push {} {}\n", symbol.kind, symbol.index))
				}
				_ => panic!("An error has occured"),
			};

			result.push_str("add\n");
			result.push_str("pop pointer 1\n");
			result.push_str("push that 0\n");
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

		let var_name = self.parse_var_name();

		let symbol = match self.find_symbol(&var_name) {
			Some(sym) => sym,
			None => panic!("An error has occured"),
		};
		match symbol.kind.as_str() {
			"field" => result.push_str(&format!("push this {}\n", symbol.index)),
			"argument" | "static" | "local" => {
				result.push_str(&format!("push {} {}\n", symbol.kind, symbol.index))
			}
			_ => panic!("An error has occured"),
		};

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

			let comma_or_else = self.peek();

			if comma_or_else.value != "," {
				return;
			};

			self.next(); // ,
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
			label_count: 0,
		}
	}

	pub fn parse(&mut self) -> String {
		self.parse_class()
	}
}
