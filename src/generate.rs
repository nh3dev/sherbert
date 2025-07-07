use comrak::{parse_document, Arena, Options};
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::nodes::{AstNode, NodeCode, NodeCodeBlock, NodeValue, NodeHtmlBlock, NodeHeading, NodeLink, NodeList};
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};

use syntect::highlighting::{ThemeSet, Theme, ThemeSettings, ThemeItem, ScopeSelector, ScopeSelectors, StyleModifier, Color, FontStyle};
use syntect::parsing::{SyntaxSetBuilder, Scope, ScopeStack};

use regex::{Regex, Captures};

use std::collections::BTreeMap;
use std::sync::{LazyLock, OnceLock};

use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

// workaround since syntax is loaded in a static and I cant be bothered to move it
static SYNTAX_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn generate(src: &Path, dst: &Path, syntax: PathBuf) {
	SYNTAX_DIR.get_or_init(|| syntax);

	let includes = |str: &str| {
		static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<include "(.*)">"#).unwrap());
		format!("{}\n", RE.replace_all(str, |c: &Captures| 
			fs::read_to_string(src.join(&c[1])).unwrap()))
	};

	let options = Options {
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
	};

	let into_html = |path: &Path| {
		let arena = Arena::new();
		let root = parse_document(&arena,
			&std::fs::read_to_string(path).unwrap(),
			&options
		);

		format!("<!DOCTYPE html>\n<html lang=en>\n<meta charset=\"UTF-8\">\n<link href=\"/theme.css\" rel=\"stylesheet\"/>\n{}\n</html>", parse_block(root))
	};

	let mkfile = |path: &Path| {
		fs::create_dir_all(path.parent().unwrap()).unwrap();
		fs::File::create(path).unwrap()
	};

	if dst.exists() { 
		fs::remove_dir_all(&*dst).unwrap(); 
	}

	fs::create_dir(&*dst).unwrap();

	for entry in walkdir::WalkDir::new(&*src) {
		let entry = entry.unwrap();
		let path = entry.path();

		if path.is_dir() { continue; }

		let dest = dst.join(path.iter().skip(1).collect::<PathBuf>());

		match path.extension().and_then(|s| s.to_str()) {
			Some("md")   => mkfile(&dest.with_extension("html")).write_all(includes(&into_html(path)).as_bytes()),
			Some("html") => mkfile(&dest).write_all(includes(&fs::read_to_string(path).unwrap()).as_bytes()),
			_            => mkfile(&dest).write_all(&fs::read(path).unwrap()),
		}.unwrap();
	}

	// println!("cargo:rerun-if-changed=files/");
}

fn parse_block<'a>(node: &'a AstNode<'a>) -> String {
	let to_string = |node: &'a AstNode<'a>|
	node.children().map(parse_block).collect();

	match node.data.borrow().value {
		NodeValue::Heading(NodeHeading { level, ..}) => {
			let str: String = to_string(node);
			format!("\n<h{level} id=\"{}\">{}</h{level}>\n", 
				str.to_lowercase().split_whitespace().collect::<String>(), str)
		},
		NodeValue::Table(..) => {
			let mut out = String::from("<table>\n");

			// Required as per GFM spec to not generate bodies
			// when a table row hasn't been seen.

			let mut seen_header = false;
			let mut seen_body   = false;

			for row in node.children() {
				if let NodeValue::TableRow(header) = row.data.borrow().value {
					if header && !seen_header {
						out.push_str("<thead>\n");
						seen_header = true;
					}

					if !seen_body && seen_header && !header {
						out.push_str("</thead>\n");
						out.push_str("<tbody>\n");
						seen_body = true;
					}

					out.push_str("<tr>\n");

					for cell in row.children() {
						if let NodeValue::TableCell = cell.data.borrow().value {
							let mut cell_str = to_string(cell);
							
							// Skip blank cells
							// if cell_str.is_empty() { continue };

							if header {
								cell_str = format!("<th>{}</th>\n", cell_str);
							} else {
								cell_str = format!("<td>{}</td>\n", cell_str);
							}

							out.push_str(cell_str.as_str());
						} else {
							eprintln!("WARNING: expected only TableCells as direct children of TableRow!");
						}
					}

					out.push_str("</tr>\n");
				} else {
					eprintln!("WARNING: expected only TableRows as direct children of Table!");
				}
			}

			if !seen_body && seen_header {
				out.push_str("</thead>\n");
			}

			out.push_str("</table>\n");
			out
		},
		// we are fancy and use em dashes, not just hyphens
		NodeValue::Text(ref text) => String::from(text).replace("--", "—"), 
		NodeValue::LineBreak      => String::from("<br>\n"),
		NodeValue::SoftBreak      => String::from(" \n"),
		NodeValue::Paragraph      => format!("<p>{}</p>\n", to_string(node)),
		NodeValue::BlockQuote     => format!("\n<div class=quote>{}</div>\n", to_string(node)),
		NodeValue::Strong         => format!("<b>{}</b>", to_string(node)),
		NodeValue::Emph           => format!("<i>{}</i>", to_string(node)),
		NodeValue::Strikethrough  => format!("<s>{}</s>", to_string(node)),
		NodeValue::HtmlInline(ref html) => String::from(html),
		NodeValue::HtmlBlock(NodeHtmlBlock { ref literal, .. }) => String::from(literal),
		NodeValue::Code(NodeCode { ref literal, .. }) => format!("<c>{literal}</c>"),
		NodeValue::Link(NodeLink { ref url, .. }) => format!("<a href=\"{url}\">{}</a>", to_string(node)), 
		NodeValue::Image(NodeLink { ref url, .. }) => format!("<img src=\"{url}\" alt=\"{}\">", to_string(node)),
		NodeValue::List(NodeList { ref bullet_char, .. }) => format!("<div class=block>\n{}</div>\n", 
			node.children()
				.map(|n| format!("<p>{}{}</p>\n", if *bullet_char as char == '-' { "- " } else { "" },
					if let Some(n) = n.children().next()
						.filter(|n| n.data.borrow().value == NodeValue::Paragraph)
					{ to_string(n) } else { parse_block(n) }))
				.collect::<String>()),
		NodeValue::CodeBlock(NodeCodeBlock { ref literal, .. }) => {
			let mut out = Vec::new();

			ADAPTER
				.write_highlighted(&mut out, Some("shard"), literal)
				.unwrap();

			let str = String::from_utf8(out).unwrap();

			static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("color:#([0-9a-f]{2})0000;").unwrap());
			let str = RE.replace_all(&str, |c: &Captures|
				format!("color:var(--colour{});", usize::from_str_radix(&c[1], 16).unwrap())
			);

			format!("<code><pre>\n{str}</pre></code>\n")
		}

		_ => to_string(node),
	}
}

static ADAPTER: LazyLock<SyntectAdapter> = LazyLock::new(|| {
	let color = |val: u8| Color {
		r: val,
		g: 0,
		b: 0,
		a: 0xff,
	};

	let item = |path: &str, hex: u8, style: Option<FontStyle>| ThemeItem {
		scope: ScopeSelectors { selectors: vec![ScopeSelector { path: ScopeStack::from_vec(vec![Scope::new(path).unwrap()]), excludes: Vec::new()}]},
		style: StyleModifier {
			foreground: Some(color(hex)),
			background: None,
			font_style: style,
		},
	};

	let theme = Theme {
		name:     Some(String::from("theme")),
		author:   None,
		settings: ThemeSettings {
			foreground: Some(color(3)),
			..Default::default()
		},
		scopes:   vec![
			item("comment",            2,  None),

			item("literal.string",     12, None),
			item("literal.char",       12, None),
			item("literal.float",      13, None),
			item("literal.integer",    13, None),

			item("keyword.control",    7,  Some(FontStyle::BOLD)),
			item("keyword.other",      7,  Some(FontStyle::BOLD)),
			item("keyword.attribute",  5,  Some(FontStyle::ITALIC)),
			item("keyword.move",       7,  Some(FontStyle::ITALIC)),

			item("entity.type",        5,  None),
         item("entity.type.named",  5,  None),
			item("entity.type.generic",5,  Some(FontStyle::BOLD)),
			item("entity.type.lesser", 5,  Some(FontStyle::ITALIC)),
			item("entity.function",    6,  None),

			item("op.special",         8,  Some(FontStyle::BOLD | FontStyle::ITALIC)),
			item("op.thread",          8,  None),
			item("op.arithmetic",      8,  None),
			item("op.bitwise",         8,  None),
			item("op.logic",           8,  None),
			item("op.ref",             5,  None),
			item("op.brackets",        4,  None),

			item("syntax.separator",   4,  None),
		],
	};

	let mut set = SyntaxSetBuilder::new();
	set.add_plain_text_syntax();
	set.add_from_folder(SYNTAX_DIR.get().unwrap(), true).unwrap();

	SyntectAdapterBuilder::new()
		.theme_set(ThemeSet {
			themes: {
				let mut themes = BTreeMap::new();
				themes.insert("theme".to_string(), theme);
				themes
			}
		})
		.theme("theme")
		.syntax_set(set.build())
		.build()
});
