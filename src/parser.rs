use serde::{Deserialize, Serialize};
use tree_sitter::{Language, Node, Parser, Tree};

/// Trait for languages that can be parsed by `docile`.
/// Implement to add support for a new format.
pub trait Parseable {
	/// The `tree-sitter` language definition.
	fn language(&self) -> Language;
}

/// A parsed document with its syntax tree.
pub struct Parsed {
	tree: Tree,
	source: String,
}

/// Generic `tree-sitter` parser for any `Parseable` language.
pub struct Treesitter {
	parser: Parser,
	language: Box<dyn Parseable>,
}

/// Byte range span within the source text.
#[derive(Debug, Serialize, Deserialize)]
pub struct Span {
	pub start: usize,
	pub end: usize,
}

impl Span {
	/// Creates a `Span` from a `Node`.
	fn from_node(node: &Node) -> Self {
		Self { start: node.start_byte(), end: node.end_byte() }
	}
}

/// Output structure representing a node in the syntax tree.
#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
	/// The node kind
	pub kind: String,

	/// The original text content of the node
	pub original: String,

	/// The text content
	pub content: String,

	/// The byte range span
	pub span: Span,

	/// Child nodes
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub children: Vec<Output>,
}

impl Treesitter {
	/// Creates a new parser for the specified `Parseable` language.
	pub fn new(language: Box<dyn Parseable>) -> Self {
		let mut parser = Parser::new();
		parser
			.set_language(&language.language())
			.expect("Treesitter language not found.");

		Self { parser, language }
	}

	/// Parses the given source into a format for programmatic input.
	pub fn parse(&mut self, source: String) -> Option<Parsed> {
		let tree = self.parser.parse(&source, None)?;

		Some(Parsed { tree, source })
	}
}

impl Parsed {
	/// Returns the root node of the syntax tree.
	pub fn root_node(&self) -> Node<'_> {
		self.tree.root_node()
	}

	/// Returns an iterator over the children of a node.
	pub fn children<'a>(&self, node: Node<'a>) -> Vec<Node<'a>> {
		let mut cursor = node.walk();
		node.children(&mut cursor).collect()
	}

	/// Returns the text content of a node.
	pub fn node_text(&self, node: &Node) -> &str {
		node.utf8_text(self.source.as_bytes()).unwrap()
	}

	/// Converts a `Node` into an `Output` structure recursively.
	pub fn to_output(&self, node: Node<'_>) -> Output {
		let original = self.node_text(&node).to_string();
		let children: Vec<Output> = self
			.children(node)
			.into_iter()
			.map(|child| self.to_output(child))
			.collect();

		Output {
			kind: node.kind().to_string(),
			content: original.clone(),
			original,
			span: Span::from_node(&node),
			children,
		}
	}

	/// Serializes the entire syntax tree to a pretty-printed JSON string.
	pub fn to_json(&self) -> Result<String, serde_json::Error> {
		let output = self.to_output(self.root_node());
		serde_json::to_string_pretty(&output)
	}
}

/// `Parseable` implementation for Markdown using `tree-sitter-md`.
pub struct Markdown;
impl Parseable for Markdown {
	fn language(&self) -> Language {
		tree_sitter_md::LANGUAGE.into()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const SOURCE: &str = r"# Foo
Bar

## Baz
Foo Bar
";

	fn assert_node(doc: &Parsed, node: Node<'_>, kind: &str, text: &str) {
		assert_eq!(node.kind(), kind);
		assert_eq!(doc.node_text(&node), text);
	}

	#[test]
	fn test_parsing() {
		let doc = Treesitter::new(Box::new(Markdown))
			.parse(SOURCE.to_string())
			.unwrap();

		let root = doc.root_node();
		assert_eq!(root.kind(), "document");

		let sections = doc.children(root);
		assert_eq!(sections.len(), 1);

		let top = sections[0];
		assert_node(&doc, top, "section", "# Foo\nBar\n\n## Baz\nFoo Bar\n");

		let top_items = doc.children(top);
		assert_eq!(top_items.len(), 3);
		assert_node(&doc, top_items[0], "atx_heading", "# Foo\n");
		assert_node(&doc, top_items[1], "paragraph", "Bar\n");

		let nested = top_items[2];
		assert_node(&doc, nested, "section", "## Baz\nFoo Bar\n");

		let nested_items = doc.children(nested);
		assert_eq!(nested_items.len(), 2);

		assert_node(&doc, nested_items[0], "atx_heading", "## Baz\n");
		assert_node(&doc, nested_items[1], "paragraph", "Foo Bar\n");
	}

	#[test]
	fn test_serialization() {
		let doc = Treesitter::new(Box::new(Markdown))
			.parse(SOURCE.to_string())
			.unwrap();

		let json = doc.to_json().unwrap();
		let parsed: Output = serde_json::from_str(&json).unwrap();

		assert_eq!(parsed.kind, "document");
		assert_eq!(parsed.original, SOURCE);
	}
}
