use comrak::{Arena, Options};
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::nodes::{AstNode, NodeCode, NodeCodeBlock, NodeFootnoteDefinition, NodeFootnoteReference, NodeHeading, NodeHtmlBlock, NodeLink, NodeList, NodeMath, NodeValue};

use regex::{Regex, Captures};

use katex::{KatexContext, Settings};

use std::sync::LazyLock;
use std::fs;
use std::path::Path;
use std::fmt::Write as _;

static COMRAK_OPTIONS: LazyLock<Options> = LazyLock::new(|| Options {
	extension: comrak::ExtensionOptions {
		strikethrough: true,
		table: true,
		superscript: true,
		footnotes: true,
		multiline_block_quotes: true,
		math_dollars: true,
		underline: true,
		spoiler: true,
		..Default::default()
	},
	..Default::default()
});


pub struct Parser {
	footnotes: Vec<(String, String)>,
	pre:  toml::Table, // kept in case we wanna add extra opts to md parsing
}

impl<'a> Parser {
	pub fn parse(path: &Path) -> Result<(toml::Table, String), toml::de::Error> {
		let file = fs::read_to_string(path).unwrap();

		let (pre, body) = match file.split_once("---\n") {
			Some((pre, body)) => (pre.parse::<toml::Table>()?, body.to_string()),
			None => (toml::Table::new(), file),
		};

		let mut this = Self { footnotes: Vec::new(), pre };

		let arena = Arena::new();
		let root = comrak::parse_document(&arena, &body, &COMRAK_OPTIONS);

		let mut out = this.parse_node(root);

		this.footnotes.into_iter().for_each(|(id, desc)|
			super::replace_first(&mut out, &format!("<preview-{id}>"), &desc));

		Ok((this.pre, out))
	}

	#[allow(clippy::wrong_self_convention)]
	fn to_string(&mut self, node: &'a AstNode<'a>) -> String {
		node.children().map(|n| self.parse_node(n)).collect()
	}

	fn parse_node(&mut self, node: &'a AstNode<'a>) -> String {
		match node.data.borrow().value {
			NodeValue::Heading(NodeHeading { level, ..}) => {
				let str: String = self.to_string(node);
				format!("\n<h{level} id=\"{}\">{}</h{level}>\n",
					str.to_lowercase().split_whitespace().collect::<String>(), str)
			},
			NodeValue::Table(..) => {
				let mut out = String::from("<table>\n");

				// Required as per GFM spec to not generate bodies when a table row hasn't been seen.
				let mut seen_header = false;
				let mut seen_body   = false;

				for row in node.children() {
					let NodeValue::TableRow(header) = row.data.borrow().value else {
						eprintln!("WARNING: expected only TableRows as direct children of Table!");
						continue;
					};

					if header && !seen_header {
						writeln!(out, "<thead>");
						seen_header = true;
					}

					if !seen_body && seen_header && !header {
						writeln!(out, "</thead>\n<tbody>");
						seen_body = true;
					}

					writeln!(out, "<tr>");

					row.children().for_each(|cell| {
						if !matches!(cell.data.borrow().value, NodeValue::TableCell) {
							eprintln!("WARNING: expected only TableCells as direct children of TableRow!");
							return;
						}

						match header {
							true => writeln!(out, "<th>{}</th>", self.to_string(cell)),
							_    => writeln!(out, "<td>{}</td>", self.to_string(cell)),
						};
					});

					writeln!(out, "</tr>");
				}

				if !seen_body && seen_header {
					writeln!(out, "</thead>");
				}

				writeln!(out, "</table>\n");
				out
			},
			// we are fancy and use em dashes, not just hyphens // AI—SLOPINATOR
			NodeValue::Text(ref text) => String::from(text).replace("--", "—"), 
			NodeValue::LineBreak      => String::from("<br>\n"),
			NodeValue::SoftBreak      => String::from(" \n"),
			NodeValue::Paragraph      => format!("<p>{}</p>\n", self.to_string(node)),
			NodeValue::BlockQuote     => format!("\n<div class=quote>{}</div>\n", self.to_string(node)),
			NodeValue::Strong         => format!("<b>{}</b>", self.to_string(node)),
			NodeValue::Emph           => format!("<i>{}</i>", self.to_string(node)),
			NodeValue::Strikethrough  => format!("<s>{}</s>", self.to_string(node)),
			NodeValue::HtmlInline(ref html) => String::from(html),
			NodeValue::HtmlBlock(NodeHtmlBlock { ref literal, .. }) => String::from(literal),
			NodeValue::Code(NodeCode { ref literal, .. }) => format!("<c>{literal}</c>"),
			NodeValue::Link(NodeLink { ref url, .. }) => format!("<a href=\"{url}\">{}</a>", self.to_string(node)), 
			NodeValue::Image(NodeLink { ref url, .. }) => format!("<figure><img src=\"{url}\" alt=\"{}\" width=\"100%\"></figure>", self.to_string(node)),
			NodeValue::List(NodeList { list_type, bullet_char, ..}) => {
				let (tag, list_style) = match list_type {
					comrak::nodes::ListType::Bullet => ("ul", format!("'{}'", char::from(bullet_char))),
					comrak::nodes::ListType::Ordered => ("ol", "decimal".into())
				};

				format!("<{} class=block style=\"list-style-type: {}\">\n{}</{}>\n", 
					tag,
					list_style,
					node.children()
						.map(|n| format!("<li>{}</li>\n", self.parse_node(n)))
						.collect::<String>(),
					tag
				)
			},

			NodeValue::CodeBlock(NodeCodeBlock { ref literal, .. }) => {
				let mut out = Vec::new();

				super::syntax::ADAPTER
					.get().unwrap()
					.write_highlighted(&mut out, Some("shard"), literal)
					.unwrap();

				let str = String::from_utf8(out).unwrap();

				// FIXME: find a better way to do this
				static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("color:#([0-9a-f]{2})0000;").unwrap());
				let str = RE.replace_all(&str, |c: &Captures|
					format!("color:var(--colour{});", usize::from_str_radix(&c[1], 16).unwrap())
				);

				format!("<code><pre>\n{str}</pre></code>\n")
			},

			NodeValue::Math(NodeMath { ref literal, .. }) => {
				static KATEX_CTX: LazyLock<KatexContext> = LazyLock::new(KatexContext::default);

				let settings = Settings::builder()
					.output(katex::OutputFormat::Mathml)
					.display_mode(false)
					.build();

				let out = katex::render_to_string(&KATEX_CTX, literal, &settings).unwrap_or_else(|e| {
					eprintln!("Failed to render math: {e}");
					format!("${literal}$")
				});

				format!("{out}\n")
			},

			NodeValue::FootnoteReference(NodeFootnoteReference { ref name, .. }) =>
				format!("<span class=\"footnote\"><sup>[{name}]</sup><span class=\"preview\"><preview-{name}></span></span>"),
			NodeValue::FootnoteDefinition(NodeFootnoteDefinition { ref name, .. }) => {
				let desc = node.children().map(|x| self.to_string(x)).collect::<String>();
				self.footnotes.push((name.clone(), desc));
				String::new()
			},
			_ => self.to_string(node)
		}
	}
}
