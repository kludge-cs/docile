use std::marker::PhantomData;

use tree_sitter::{Language, Node, Parser, Tree};

/// Trait for languages that can be parsed by `docile`.
/// Implement to add support for a new format.
pub trait Parseable {
	/// The `tree-sitter` language definition.
	fn language() -> Language;
}

/// A parsed document with its syntax tree.
pub struct Parsed<P: Parseable> {
	tree: Tree,
	source: String,
	_marker: PhantomData<P>,
}

/// Generic `tree-sitter` parser for any `Parseable` language.
pub struct Treesitter<P: Parseable> {
	parser: Parser,
	_marker: PhantomData<P>,
}

impl<P: Parseable> Treesitter<P> {
	/// Creates a new parser for the specified `Parseable` language.
	pub fn new() -> Self {
		let mut parser = Parser::new();
		parser
			.set_language(&P::language())
			.expect("Treesitter language not found.");

		Self { parser, _marker: PhantomData }
	}

	/// Parses the given source into a format for programmatic input.
	pub fn parse(&mut self, source: String) -> Option<Parsed<P>> {
		let tree = self.parser.parse(&source, None)?;

		Some(Parsed { tree, source, _marker: PhantomData })
	}
}

impl<P: Parseable> Parsed<P> {
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
		&self.source[node.byte_range()]
	}
}

/// `Parseable` implementation for Markdown using `tree-sitter-md`.
pub struct Markdown;
impl Parseable for Markdown {
	fn language() -> Language {
		tree_sitter_md::LANGUAGE.into()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn assert_node<P: Parseable>(
		doc: &Parsed<P>,
		node: Node<'_>,
		kind: &str,
		text: &str,
	) {
		assert_eq!(node.kind(), kind);
		assert_eq!(doc.node_text(&node), text);
	}

	#[test]
	fn test_parsing() {
		let src = r"# Foo
Bar

## Baz
Foo Bar
";
		let doc = Treesitter::<Markdown>::new().parse(src).unwrap();

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
}
