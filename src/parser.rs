use crate::tokenizer::{Token, TokenType};

use crate::tree::{Leaf, Node, Tree, TreeElement};
use std::collections::VecDeque;

pub struct Parser {
	tokens: VecDeque<Token>,
}

impl Parser {
	fn next(&mut self) -> Token {
		self.tokens.pop_front().unwrap()
	}

	fn peek(&mut self) -> Token {
		self.tokens.front().unwrap().clone()
	}

	fn parse_class(&mut self) -> TreeElement {
		let mut class_node = Node::new(String::from("class"));
		// Class token
		class_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword"),
			self.next().value,
		)));
		// Class name
		class_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("identifier"),
			self.next().value,
		)));
		// Opening curly brace
		class_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));
		// Optional class variables declaration
		loop {
			let next_token = self.peek();
			if next_token.value != "static" && next_token.value != "field" {
				break;
			};
			class_node.add_child(self.parse_class_var_dec());
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
			class_node.add_child(self.parse_subroutine_dec());
		}
		// Closing curly brace
		class_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		TreeElement::Node(class_node)
	}

	fn parse_subroutine_dec(&mut self) -> TreeElement {
		let mut subrountine_dec_node = Node::new(String::from("subroutine_dec"));

		subrountine_dec_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword"),
			self.next().value,
		)));

		let void_or_type = self.peek();

		if void_or_type.value == "void" {
			subrountine_dec_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("keyword"),
				self.next().value,
			)));
		} else {
			subrountine_dec_node.add_child(self.parse_type());
		};

		subrountine_dec_node.add_child(self.parse_subroutine_name());

		subrountine_dec_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		match self.parse_parameter_list() {
			Some(elem) => subrountine_dec_node.add_child(elem),
			None => {}
		};

		subrountine_dec_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		subrountine_dec_node.add_child(self.parse_subroutine_body());

		TreeElement::Node(subrountine_dec_node)
	}

	fn parse_subroutine_name(&mut self) -> TreeElement {
		let mut subroutine_name_node = Node::new(String::from("subroutine_name"));
		subroutine_name_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("identifier"),
			self.next().value,
		)));
		TreeElement::Node(subroutine_name_node)
	}

	fn parse_subroutine_body(&mut self) -> TreeElement {
		let mut subroutine_body_node = Node::new(String::from("subroutine_body"));

		subroutine_body_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		loop {
			let var_or_else = self.peek();

			if var_or_else.value != "var" {
				break;
			}

			subroutine_body_node.add_child(self.parse_var_dec())
		}

		match self.parse_statements() {
			Some(elem) => subroutine_body_node.add_child(elem),
			None => {}
		};

		subroutine_body_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		TreeElement::Node(subroutine_body_node)
	}

	fn parse_var_dec(&mut self) -> TreeElement {
		let mut var_dec_node = Node::new(String::from("var_dec"));

		var_dec_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword"),
			self.next().value,
		)));

		var_dec_node.add_child(self.parse_type());

		var_dec_node.add_child(self.parse_var_name());

		loop {
			let next_token = self.peek();

			if next_token.value == ";" {
				var_dec_node.add_child(TreeElement::Leaf(Leaf::new(
					String::from("symbol"),
					self.next().value,
				)));
				return TreeElement::Node(var_dec_node);
			}

			var_dec_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));

			var_dec_node.add_child(self.parse_var_name());
		}
	}

	fn parse_statements(&mut self) -> Option<TreeElement> {
		match self.peek().value.as_str() {
			"let" | "if" | "while" | "do" | "return" => {
				let mut statements_node = Node::new(String::from("statements"));
				loop {
					let next_token = self.peek();

					if next_token.value != "let"
						&& next_token.value != "if"
						&& next_token.value != "while"
						&& next_token.value != "do"
						&& next_token.value != "return"
					{
						return Some(TreeElement::Node(statements_node));
					}

					statements_node.add_child(self.parse_statement());
				}
			}
			_ => None,
		}
	}

	fn parse_statement(&mut self) -> TreeElement {
		match self.peek().value.as_str() {
			"let" => self.parse_let_statement(),
			"if" => self.parse_if_statement(),
			"while" => self.parse_while_statement(),
			"do" => self.parse_do_statement(),
			"return" => self.parse_return_statement(),
			_ => panic!("An error has occured"),
		}
	}

	fn parse_let_statement(&mut self) -> TreeElement {
		let mut let_statement_node = Node::new(String::from("let_statement"));

		let_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword"),
			self.next().value,
		)));

		let_statement_node.add_child(self.parse_var_name());

		if self.peek().value == "[" {
			let_statement_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));
			let_statement_node.add_child(self.parse_expression());
			let_statement_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));
		}

		let_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));
		let_statement_node.add_child(self.parse_expression());

		let_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		TreeElement::Node(let_statement_node)
	}

	fn parse_if_statement(&mut self) -> TreeElement {
		let mut if_statement_node = Node::new(String::from("if_statement"));

		if_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword"),
			self.next().value,
		)));

		if_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));
		if_statement_node.add_child(self.parse_expression());

		if_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		if_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));
		match self.parse_statements() {
			Some(elem) => if_statement_node.add_child(elem),
			None => {}
		};

		if_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		if self.peek().value == "else" {
			if_statement_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("keyword"),
				self.next().value,
			)));

			if_statement_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));

			match self.parse_statements() {
				Some(elem) => if_statement_node.add_child(elem),
				None => {}
			};

			if_statement_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));
		}

		TreeElement::Node(if_statement_node)
	}

	fn parse_while_statement(&mut self) -> TreeElement {
		let mut while_statement_node = Node::new(String::from("while_statement"));

		while_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword"),
			self.next().value,
		)));

		while_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));
		while_statement_node.add_child(self.parse_expression());

		while_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		while_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));
		match self.parse_statements() {
			Some(elem) => while_statement_node.add_child(elem),
			None => {}
		};

		while_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		TreeElement::Node(while_statement_node)
	}

	fn parse_do_statement(&mut self) -> TreeElement {
		let mut do_statement_node = Node::new(String::from("do_statement"));

		do_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword"),
			self.next().value,
		)));

		do_statement_node.add_child(self.parse_subroutine_call());

		do_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		TreeElement::Node(do_statement_node)
	}

	fn parse_return_statement(&mut self) -> TreeElement {
		let mut return_statement_node = Node::new(String::from("return_statement"));

		return_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword"),
			self.next().value,
		)));

		if self.peek().value != ";" {
			return_statement_node.add_child(self.parse_expression());
		};

		return_statement_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		TreeElement::Node(return_statement_node)
	}

	fn parse_subroutine_call(&mut self) -> TreeElement {
		let mut subroutine_call_node = Node::new(String::from("subroutine_call"));

		let func_or_class = self.next();

		match self.peek().value.as_str() {
			"." => {
				self.tokens.insert(0, func_or_class);
				subroutine_call_node.add_child(self.parse_class_name());
				subroutine_call_node.add_child(TreeElement::Leaf(Leaf::new(
					String::from("symbol"),
					self.next().value,
				)));
				subroutine_call_node.add_child(self.parse_subroutine_name());
			}
			_ => {
				subroutine_call_node.add_child(self.parse_subroutine_name());
			}
		};

		subroutine_call_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		if self.peek().value != ")" {
			subroutine_call_node.add_child(self.parse_expression_list());
		};

		subroutine_call_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));

		TreeElement::Node(subroutine_call_node)
	}

	fn parse_expression_list(&mut self) -> TreeElement {
		let mut expression_list_node = Node::new(String::from("expression_list"));

		loop {
			expression_list_node.add_child(self.parse_expression());

			if self.peek().value != "," {
				return TreeElement::Node(expression_list_node);
			};

			expression_list_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));
		}
	}

	fn parse_expression(&mut self) -> TreeElement {
		let mut expression_node = Node::new(String::from("expression"));

		expression_node.add_child(self.parse_term());

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
				return TreeElement::Node(expression_node);
			}

			expression_node.add_child(self.parse_op());
			expression_node.add_child(self.parse_term());
		}
	}

	fn parse_op(&mut self) -> TreeElement {
		let mut op_node = Node::new(String::from("op"));
		op_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("symbol"),
			self.next().value,
		)));
		TreeElement::Node(op_node)
	}

	fn parse_term(&mut self) -> TreeElement {
		let mut term_node = Node::new(String::from("term"));

		let next_token = self.peek();

		if next_token.token == TokenType::IntegerConstant {
			term_node.add_child(self.parse_integer_constant());
			return TreeElement::Node(term_node);
		};

		if next_token.token == TokenType::StringConstant {
			term_node.add_child(self.parse_string_constant());
			return TreeElement::Node(term_node);
		};

		if next_token.value == "true"
			|| next_token.value == "false"
			|| next_token.value == "null"
			|| next_token.value == "this"
		{
			term_node.add_child(self.parse_keyword_constant());
			return TreeElement::Node(term_node);
		};

		if next_token.value == "(" {
			term_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));
			term_node.add_child(self.parse_expression());
			term_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));
			return TreeElement::Node(term_node);
		};

		if next_token.value == "-" || next_token.value == "~" {
			term_node.add_child(self.parse_unary_op());
			term_node.add_child(self.parse_term());
			return TreeElement::Node(term_node);
		};

		// Var name or subroutine call
		let var_name_or_sub_name = self.next();

		let bracket_or_else = self.peek();

		// Var[]
		if bracket_or_else.value == "[" {
			term_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("identifier"),
				var_name_or_sub_name.value,
			)));
			term_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));
			term_node.add_child(self.parse_expression());
			term_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));
			return TreeElement::Node(term_node);
		};

		// Subroutine
		if bracket_or_else.value == "(" || bracket_or_else.value == "." {
			self.tokens.insert(0, var_name_or_sub_name);
			term_node.add_child(self.parse_subroutine_call());
			return TreeElement::Node(term_node);
		};

		// Var name
		self.tokens.insert(0, var_name_or_sub_name);

		term_node.add_child(self.parse_var_name());

		return TreeElement::Node(term_node);
	}

	fn parse_unary_op(&mut self) -> TreeElement {
		TreeElement::Leaf(Leaf::new(String::from("unary_op"), self.next().value))
	}

	fn parse_integer_constant(&mut self) -> TreeElement {
		let mut integer_constant_node = Node::new(String::from("integer_constant"));
		integer_constant_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("integer_constant"),
			self.next().value,
		)));
		TreeElement::Node(integer_constant_node)
	}

	fn parse_string_constant(&mut self) -> TreeElement {
		let mut string_constant_node = Node::new(String::from("string_constant"));
		string_constant_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("string_constant"),
			self.next().value,
		)));
		TreeElement::Node(string_constant_node)
	}

	fn parse_keyword_constant(&mut self) -> TreeElement {
		let mut keyword_constant_node = Node::new(String::from("keyword_constant"));
		keyword_constant_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword_constant"),
			self.next().value,
		)));
		TreeElement::Node(keyword_constant_node)
	}

	fn parse_parameter_list(&mut self) -> Option<TreeElement> {
		let type_or_else = self.peek();

		if type_or_else.value != "int"
			&& type_or_else.value != "char"
			&& type_or_else.value != "boolean"
			&& type_or_else.token != TokenType::Identifier
		{
			return None;
		};

		let mut parameter_list_node = Node::new(String::from("parameter_list"));

		loop {
			parameter_list_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("type"),
				self.next().value,
			)));
			parameter_list_node.add_child(self.parse_var_name());

			let comma_or_else = self.next();

			if comma_or_else.value != "," {
				return Some(TreeElement::Node(parameter_list_node));
			};

			parameter_list_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				self.next().value,
			)));
		}
	}

	fn parse_class_var_dec(&mut self) -> TreeElement {
		let mut class_var_dec_node = Node::new(String::from("class_var_dec"));

		class_var_dec_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("keyword"),
			self.next().value,
		)));

		class_var_dec_node.add_child(self.parse_type());

		loop {
			class_var_dec_node.add_child(self.parse_var_name());

			// Check for other variable declarations
			let comma_or_semi = self.next();

			class_var_dec_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("symbol"),
				comma_or_semi.value.clone(),
			)));

			if comma_or_semi.value == ";" {
				return TreeElement::Node(class_var_dec_node);
			}
		}
	}

	fn parse_var_name(&mut self) -> TreeElement {
		let mut var_name_node = Node::new(String::from("var_name"));
		var_name_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("identifier"),
			self.next().value,
		)));
		TreeElement::Node(var_name_node)
	}

	fn parse_type(&mut self) -> TreeElement {
		let mut type_node = Node::new(String::from("type"));

		let front_token = self.peek();

		if front_token.value == "int" || front_token.value == "char" || front_token.value == "boolean" {
			type_node.add_child(TreeElement::Leaf(Leaf::new(
				String::from("keyword"),
				self.next().value,
			)));
		} else {
			type_node.add_child(self.parse_class_name())
		};

		TreeElement::Node(type_node)
	}

	fn parse_class_name(&mut self) -> TreeElement {
		let mut class_name_node = Node::new(String::from("class_name"));
		class_name_node.add_child(TreeElement::Leaf(Leaf::new(
			String::from("identifier"),
			self.next().value,
		)));
		TreeElement::Node(class_name_node)
	}

	pub fn new(tokens: VecDeque<Token>) -> Parser {
		Parser { tokens: tokens }
	}

	pub fn parse(&mut self) -> Tree {
		let root = self.parse_class();
		Tree::new(root)
	}
}
