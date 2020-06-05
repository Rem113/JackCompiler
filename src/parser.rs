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
	class_symbol_table: HashMap<String, Symbol>,
	func_symbol_table: HashMap<String, Symbol>,
}

impl Parser {
	fn find_symbol(&self, map: &HashMap<String, Symbol>, name: String) -> Symbol {
		map.get(&name).unwrap().clone()
	}

	fn find_symbol_in_class(&self, name: String) -> Symbol {
		self.find_symbol(&self.class_symbol_table, name)
	}

	fn new_func_symbol_table(&mut self) {
		self.func_symbol_table = HashMap::new();
	}

	fn find_symbol_in_func(&self, name: String) -> Symbol {
		self.find_symbol(&self.func_symbol_table, name)
	}

	fn add_symbol_in_class(&mut self, name: String, kind: String, typing: String) {
		let mut index = 0;
		let same_kind: Vec<(&String, &Symbol)> = self
			.class_symbol_table
			.iter()
			.filter(|(_, sym)| sym.kind == kind)
			.collect();
		if same_kind.len() != 0 {
			let (_, max_sym) = same_kind
				.iter()
				.max_by(|(_, sym1), (_, sym2)| sym1.index.cmp(&sym2.index))
				.unwrap();
			index = max_sym.index + 1;
		};
		self.class_symbol_table.insert(
			name,
			Symbol {
				kind: kind,
				typing: typing,
				index: index,
			},
		);
	}

	fn add_symbol_in_func(&mut self, name: String, kind: String, typing: String) {
		let mut index = 0;
		let same_kind: Vec<(&String, &Symbol)> = self
			.func_symbol_table
			.iter()
			.filter(|(_, sym)| sym.kind == kind)
			.collect();
		if same_kind.len() != 0 {
			let (_, max_sym) = same_kind
				.iter()
				.max_by(|(_, sym1), (_, sym2)| sym1.index.cmp(&sym2.index))
				.unwrap();
			index = max_sym.index + 1;
		};
		self.func_symbol_table.insert(
			name,
			Symbol {
				kind: kind,
				typing: typing,
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
		let mut result = String::from("<class>\n");
		result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));
		result.push_str(&format!("<identifier>{}</identifier>\n", self.next().value));
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

		// Optional class variables declaration
		loop {
			let next_token = self.peek();
			if next_token.value != "static" && next_token.value != "field" {
				break;
			};
			result.push_str(&self.parse_class_var_dec());
		}

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
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str("</class>\n");
		result
	}

	fn parse_subroutine_dec(&mut self) -> String {
		let mut result = String::from("<subroutine_dec>\n");
		result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));

		let void_or_type = self.peek();

		if void_or_type.value == "void" {
			result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));
		} else {
			result.push_str(&self.parse_type());
		};

		result.push_str(&self.parse_subroutine_name());
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

		match self.parse_parameter_list() {
			Some(elem) => result.push_str(&elem),
			None => {}
		};

		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str(&self.parse_subroutine_body());
		result.push_str("</subroutine_dec>\n");
		result
	}

	fn parse_subroutine_name(&mut self) -> String {
		let mut result = String::from("<subroutine_name>\n");
		result.push_str(&format!("<identifier>{}</identifier>\n", self.next().value));
		result.push_str("</subroutine_name>\n");
		result
	}

	fn parse_subroutine_body(&mut self) -> String {
		let mut result = String::from("<subroutine_body>\n");
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

		loop {
			let var_or_else = self.peek();

			if var_or_else.value != "var" {
				break;
			}

			result.push_str(&self.parse_var_dec())
		}

		match self.parse_statements() {
			Some(elem) => result.push_str(&elem),
			None => {}
		};

		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str("</subroutine_body>\n");
		result
	}

	fn parse_var_dec(&mut self) -> String {
		let mut result = String::from("<var_dec>\n");
		result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));
		result.push_str(&self.parse_type());
		result.push_str(&self.parse_var_name());

		loop {
			let next_token = self.peek();

			if next_token.value == ";" {
				result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
				result.push_str("</var_dec>\n");
				return result;
			}

			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
			result.push_str(&self.parse_var_name());
		}
	}

	fn parse_statements(&mut self) -> Option<String> {
		match self.peek().value.as_str() {
			"let" | "if" | "while" | "do" | "return" => {
				let mut result = String::from("<statements>\n");
				loop {
					let next_token = self.peek();

					if next_token.value != "let"
						&& next_token.value != "if"
						&& next_token.value != "while"
						&& next_token.value != "do"
						&& next_token.value != "return"
					{
						result.push_str("</statements>\n");
						return Some(result);
					}

					result.push_str(&self.parse_statement());
				}
			}
			_ => None,
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
		let mut result = String::from("<let_statement>\n");
		result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));
		result.push_str(&self.parse_var_name());

		if self.peek().value == "[" {
			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
			result.push_str(&self.parse_expression());
			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		}

		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str(&self.parse_expression());
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str("</let_statement>\n");
		result
	}

	fn parse_if_statement(&mut self) -> String {
		let mut result = String::from("<if_statement>\n");

		result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str(&self.parse_expression());
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

		match self.parse_statements() {
			Some(elem) => result.push_str(&elem),
			None => {}
		};

		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

		if self.peek().value == "else" {
			result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));
			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

			match self.parse_statements() {
				Some(elem) => result.push_str(&elem),
				None => {}
			};

			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		}

		result.push_str("</if_statement>\n");
		result
	}

	fn parse_while_statement(&mut self) -> String {
		let mut result = String::from("<while_statement>\n");
		result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str(&self.parse_expression());
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

		match self.parse_statements() {
			Some(elem) => result.push_str(&elem),
			None => {}
		};
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str("</while_statement>\n");
		result
	}

	fn parse_do_statement(&mut self) -> String {
		let mut result = String::from("<do_statement>\n");
		result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));
		result.push_str(&self.parse_subroutine_call());
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str("</do_statement>\n");
		result
	}

	fn parse_return_statement(&mut self) -> String {
		let mut result = String::from("<return_statement>\n");
		result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));

		if self.peek().value != ";" {
			result.push_str(&self.parse_expression());
		};

		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str("</return_statement>\n");
		result
	}

	fn parse_subroutine_call(&mut self) -> String {
		let mut result = String::from("<subroutine_call>\n");
		let func_or_class = self.next();

		match self.peek().value.as_str() {
			"." => {
				self.tokens.insert(0, func_or_class);
				result.push_str(&self.parse_class_name());
				result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
				result.push_str(&self.parse_subroutine_name());
			}
			_ => {
				result.push_str(&self.parse_subroutine_name());
			}
		};

		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

		if self.peek().value != ")" {
			result.push_str(&self.parse_expression_list());
		};

		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str("</subroutine_call>\n");
		result
	}

	fn parse_expression_list(&mut self) -> String {
		let mut result = String::from("<expression_list>\n");

		loop {
			result.push_str(&self.parse_expression());

			if self.peek().value != "," {
				result.push_str("</expression_list>\n");
				return result;
			};

			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		}
	}

	fn parse_expression(&mut self) -> String {
		let mut result = String::from("<expression>\n");
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
				result.push_str("</expression>\n");
				return result;
			}

			result.push_str(&self.parse_op());
			result.push_str(&self.parse_term());
		}
	}

	fn parse_op(&mut self) -> String {
		let mut result = String::from("<op>\n");
		result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		result.push_str("</op>\n");
		result
	}

	fn parse_term(&mut self) -> String {
		let mut result = String::from("<term>\n");

		let next_token = self.peek();

		if next_token.token == TokenType::IntegerConstant {
			result.push_str(&self.parse_integer_constant());
			result.push_str("</term>\n");
			return result;
		};
		if next_token.token == TokenType::StringConstant {
			result.push_str(&self.parse_string_constant());
			result.push_str("</term>\n");
			return result;
		};
		if next_token.value == "true"
			|| next_token.value == "false"
			|| next_token.value == "null"
			|| next_token.value == "this"
		{
			result.push_str(&self.parse_keyword_constant());
			result.push_str("</term>\n");
			return result;
		};

		if next_token.value == "(" {
			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

			result.push_str(&self.parse_expression());
			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
			result.push_str("</term>\n");
			return result;
		};

		if next_token.value == "-" || next_token.value == "~" {
			result.push_str(&self.parse_unary_op());
			result.push_str(&self.parse_term());
			result.push_str("</op>\n");
			return result;
		};

		// Var name or subroutine call
		let var_name_or_sub_name = self.next();

		let bracket_or_else = self.peek();

		// Var[]
		if bracket_or_else.value == "[" {
			result.push_str(&format!(
				"<identifier>{}</identifier>\n",
				var_name_or_sub_name.value
			));
			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

			result.push_str(&self.parse_expression());
			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
			result.push_str("</term>\n");
			return result;
		};

		// Subroutine
		if bracket_or_else.value == "(" || bracket_or_else.value == "." {
			self.tokens.insert(0, var_name_or_sub_name);
			result.push_str(&self.parse_subroutine_call());
			result.push_str("</term>\n");
			return result;
		};

		// Var name
		self.tokens.insert(0, var_name_or_sub_name);

		result.push_str(&self.parse_var_name());
		result.push_str("</term>\n");
		return result;
	}

	fn parse_unary_op(&mut self) -> String {
		String::from(format!("<unary_op>{}</unary_op>\n", self.next().value))
	}

	fn parse_integer_constant(&mut self) -> String {
		String::from(format!(
			"<integer_constant>{}</integer_constant>\n",
			self.next().value
		))
	}

	fn parse_string_constant(&mut self) -> String {
		String::from(format!(
			"<string_constant>{}</string_constant>\n",
			self.next().value
		))
	}

	fn parse_keyword_constant(&mut self) -> String {
		String::from(format!(
			"<keyword_constant>{}</keyword_constant>\n",
			self.next().value
		))
	}

	fn parse_parameter_list(&mut self) -> Option<String> {
		let type_or_else = self.peek();

		if type_or_else.value != "int"
			&& type_or_else.value != "char"
			&& type_or_else.value != "boolean"
			&& type_or_else.token != TokenType::Identifier
		{
			return None;
		};

		let mut result = String::from("<parameter_list>\n");

		loop {
			result.push_str(&format!("<type>{}</type>\n", self.next().value));
			result.push_str(&self.parse_var_name());

			let comma_or_else = self.next();

			if comma_or_else.value != "," {
				result.push_str("</parameter_list>\n");
				return Some(result);
			};

			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));
		}
	}

	fn parse_class_var_dec(&mut self) -> String {
		let mut result = String::from("<class_var_dec>\n");
		result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));

		result.push_str(&self.parse_type());

		loop {
			result.push_str(&self.parse_var_name());

			// Check for other variable declarations
			let comma_or_semi = self.next();

			result.push_str(&format!("<symbol>{}</symbol>\n", self.next().value));

			if comma_or_semi.value == ";" {
				result.push_str("</class_var_dec>\n");
				return result;
			}
		}
	}

	fn parse_var_name(&mut self) -> String {
		let mut result = String::from("<var_name>\n");
		result.push_str(&format!("<identifier>{}</identifier>\n", self.next().value));
		result.push_str("</var_name>\n");
		result
	}

	fn parse_type(&mut self) -> String {
		let mut result = String::from("<type>\n");

		let front_token = self.peek();

		if front_token.value == "int" || front_token.value == "char" || front_token.value == "boolean" {
			result.push_str(&format!("<keyword>{}</keyword>\n", self.next().value));
		} else {
			result.push_str(&self.parse_class_name())
		};

		result.push_str("</type>\n");
		result
	}

	fn parse_class_name(&mut self) -> String {
		let mut result = String::from("<class_name>\n");
		result.push_str(&format!("<identifier>{}</identifier>\n", self.next().value));
		result.push_str("</class_name>\n");
		result
	}

	pub fn new(tokens: VecDeque<Token>) -> Parser {
		Parser {
			tokens: tokens,
			class_symbol_table: HashMap::new(),
			func_symbol_table: HashMap::new(),
		}
	}

	pub fn parse(&mut self) -> String {
		self.parse_class()
	}
}
