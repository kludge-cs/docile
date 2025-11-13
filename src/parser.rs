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
	pub fn parse(&mut self, source: &str) -> Option<Parsed<P>> {
		let tree = self.parser.parse(source, None)?;
		Some(Parsed { tree, source: source.to_string(), _marker: PhantomData })
	}
}

impl<P: Parseable> Parsed<P> {
	/// Returns the root node of the syntax tree.
	pub fn root_node(&self) -> Node<'_> {
		self.tree.root_node()
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

		// TODO: Write some helper, be it a macro or such, to make these tests
		// less verbose.
		let mut cursor = root.walk();
		let children: Vec<_> = root.children(&mut cursor).collect();
		assert_eq!(children.len(), 1);

		let top = children[0];
		assert_eq!(children[0].kind(), "section");
		assert_eq!(doc.node_text(&top), "# Foo\nBar\n\n## Baz\nFoo Bar\n");

		let mut top_cursor = top.walk();
		let top_items: Vec<_> = top.children(&mut top_cursor).collect();
		assert_eq!(top_items.len(), 3);

		assert_eq!(top_items[0].kind(), "atx_heading");
		assert_eq!(doc.node_text(&top_items[0]), "# Foo\n");

		assert_eq!(top_items[1].kind(), "paragraph");
		assert_eq!(doc.node_text(&top_items[1]), "Bar\n");

		let nested_section = top_items[2];
		assert_eq!(nested_section.kind(), "section");
		assert_eq!(doc.node_text(&nested_section), "## Baz\nFoo Bar\n");

		let mut nested_cursor = nested_section.walk();
		let nested_items: Vec<_> =
			nested_section.children(&mut nested_cursor).collect();
		assert_eq!(nested_items.len(), 2);

		assert_eq!(nested_items[0].kind(), "atx_heading");
		assert_eq!(doc.node_text(&nested_items[0]), "## Baz\n");

		assert_eq!(nested_items[1].kind(), "paragraph");
		assert_eq!(doc.node_text(&nested_items[1]), "Foo Bar\n");
	}
}
