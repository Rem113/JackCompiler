#[derive(Clone)]
pub enum TreeElement {
	Node(Node),
	Leaf(Leaf),
}

#[derive(Clone)]
pub struct Node {
	name: String,
	children: Vec<TreeElement>,
}

impl Node {
	pub fn new(name: String) -> Node {
		Node {
			name: name,
			children: vec![],
		}
	}

	pub fn add_child(&mut self, element: TreeElement) {
		self.children.push(element);
	}

	fn to_xml(&self) -> String {
		let mut result = String::new();

		result.push_str(&format!("<{}>\n", self.name));

		self.children.iter().for_each(|child| match child {
			TreeElement::Node(node) => result.push_str(node.to_xml_indent(1).as_str()),
			TreeElement::Leaf(leaf) => result.push_str(leaf.to_xml_indent(1).as_str()),
		});

		result.push_str(&format!("</{}>\n", self.name));

		result
	}

	fn to_xml_indent(&self, indent: usize) -> String {
		let mut result = String::new();

		for _ in 0..indent {
			result.push_str("  ");
		}

		result.push_str(&format!("<{}>\n", self.name));

		self.children.iter().for_each(|child| match child {
			TreeElement::Node(node) => result.push_str(node.to_xml_indent(indent + 1).as_str()),
			TreeElement::Leaf(leaf) => result.push_str(leaf.to_xml_indent(indent + 1).as_str()),
		});

		for _ in 0..indent {
			result.push_str("  ");
		}

		result.push_str(&format!("</{}>\n", self.name));

		result
	}
}

#[derive(Clone)]
pub struct Leaf {
	name: String,
	value: String,
}

impl Leaf {
	pub fn new(name: String, value: String) -> Leaf {
		Leaf {
			name: name,
			value: value,
		}
	}

	fn to_xml(&self) -> String {
		format!("<{}>{}</{}>\n", self.name, self.value, self.name)
	}

	fn to_xml_indent(&self, indent: usize) -> String {
		let mut result = String::new();

		for _ in 0..indent {
			result.push_str("  ");
		}

		result.push_str(self.to_xml().as_str());

		result
	}
}

#[derive(Clone)]
pub struct Tree {
	root: TreeElement,
}

impl Tree {
	pub fn new(root: TreeElement) -> Tree {
		Tree { root: root }
	}

	pub fn to_xml(&self) -> String {
		match self.clone().root {
			TreeElement::Node(node) => node.to_xml(),
			TreeElement::Leaf(leaf) => leaf.to_xml(),
		}
	}
}
