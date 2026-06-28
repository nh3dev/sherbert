use comrak::{Arena, Options};
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::nodes::{AstNode, NodeCode, NodeCodeBlock, NodeFootnoteDefinition, NodeFootnoteReference, NodeHeading, NodeHtmlBlock, NodeLink, NodeList, NodeMath, NodeValue};
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};

use syntect::highlighting::{ThemeSet, Theme, ThemeSettings, ThemeItem, ScopeSelector, ScopeSelectors, StyleModifier, Color, FontStyle};
use syntect::parsing::{SyntaxSetBuilder, Scope, ScopeStack};

use regex::{Regex, Captures};

use katex::{KatexContext, Settings};

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::{LazyLock, OnceLock};

use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write as _;
use std::fmt::Write as _;

static INCLUDE_RE:    LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<include "(.*)">"#).unwrap());
static BLOG_ENTRY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<blog-header "(.*)", "(.*)", "(.*)">"#).unwrap());

pub fn generate(src: &Path, dst: &Path, syntax: PathBuf) {
	const BLOG_DIR:   &str = "blog";
	const ASSETS_DIR: &str = "assets";

	SYNTAX_DIR.get_or_init(|| syntax);

	if dst.exists() { 
		fs::remove_dir_all(dst).unwrap(); 
	}

	fs::create_dir(dst).unwrap();

	let blog_dir = src.join(BLOG_DIR);

	let resolve_macros = |mut i: String| {
		let f  = |c: &Captures| fs::read_to_string(src.join(&c[1])).unwrap();
		if let Cow::Owned(s) = INCLUDE_RE.replace_all(&i, f) { i = s; }

		let f  = |c: &Captures| format!(r#"<div class="blog-header"><h2>{}</h2><p>by {}</b><p class="blog-date">{}</p></div>"#, &c[1], &c[2], &c[3]);
		if let Cow::Owned(s) = BLOG_ENTRY_RE.replace_all(&i, f) { i = s; }

		i
	};

	//
	// process all files
	walkdir::WalkDir::new(src).into_iter()
		.filter_map(Result::ok).filter(|e| !e.path().is_dir()).for_each(|e| {
			let path = e.path();
			let dest = dst.join(path.iter().skip(1).collect::<PathBuf>());

			if path.starts_with(&blog_dir) && e.file_name() == ASSETS_DIR { return; }

			match path.extension().and_then(|s| s.to_str()) {
				Some("md")   => mkfile(&dest.with_extension("html")).write_all(resolve_macros(Parser::parse(path)).as_bytes()),
				Some("html") => mkfile(&dest).write_all(resolve_macros(fs::read_to_string(path).unwrap()).as_bytes()),
				_            => mkfile(&dest).write_all(&fs::read(path).unwrap()),
			}.unwrap();
		});

	//
	// blogs
	let mut posts = std::fs::read_dir(&blog_dir).unwrap().filter_map(|e| {
			let path = e.ok()?.path();

			if path.file_name()? == "index.md" { return None; }

			let content = std::fs::read_to_string(&path).ok()?;
			let c = BLOG_ENTRY_RE.captures(&content)?;
			let (name, auth, date) = (c[1].to_string(), c[2].to_string(), c[3].to_string());

			let content = Parser::parse(&path);
			let newpath = dst.join(path.iter().skip(1).collect::<PathBuf>()).with_extension("html");
			mkfile(&newpath).write_all(resolve_macros(content.clone()).as_bytes());

			Some((content, name, auth, date, newpath))
		}).collect::<Vec<_>>();

	posts.sort_unstable_by_key(|(_, _, _, s, _)| 
		chrono::NaiveDate::parse_from_str(s, "%d-%m-%Y").unwrap());

	let latest = posts.last().map_or_else(String::new, 
		|p| INCLUDE_RE.replace_all(&p.0, |_: &Captures| "").into_owned());

	let blog_list = match posts.as_slice() {
		[] => String::from("<s>No posts found :(</s>"),
		_ => {
			let mut out = String::from("<div class=\"block\">");

			posts.into_iter().rev().for_each(|(_, name, _, date, path)| 
				writeln!(out, "<p><a href=\"{}\">{name} - {date}</a></p>",
					path.iter().skip(1).collect::<PathBuf>().display()).unwrap());

			write!(out, "</div>");
			out
		},
	};

	let blog_idx = Parser::parse(&blog_dir.join("index.md"))
		.replacen("<latest>", &latest, 1)
		.replacen("<allposts>", &blog_list, 1);

	mkfile(&dst.join(BLOG_DIR).join("index.html")).write_all(resolve_macros(blog_idx).as_bytes());
}


fn mkfile(path: &Path) -> fs::File {
	fs::create_dir_all(path.parent().unwrap()).unwrap();
	fs::File::create(path).unwrap()
}

struct Parser<'p> {
	footnotes: Vec<(String, Option<String>)>,
	path: &'p Path
}

impl<'a, 'p> Parser<'p> {
	fn parse(path: &'p Path) -> String {
		static OPTIONS: LazyLock<Options> = LazyLock::new(|| Options {
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

		let arena = Arena::new();
		let root = comrak::parse_document(&arena, &fs::read_to_string(path).unwrap(), &OPTIONS);

		let mut this = Self { footnotes: Vec::new(), path };
		let mut out = this.parse_node(root);

		if let Some(marker) = out.find("<footnotes>") && !this.footnotes.is_empty() {
			let footnotes = this.footnotes.iter().fold(String::new(), |s, (name, desc)| {
				let Some(desc) = desc else {
					println!("WARN: [{}] missing footnote description for ref `{name}`", this.path.display());
					return s;
				};

				format!("{s}\n<p class=\"footnote-def\" id=\"footnote-{name}\">{name}: {desc}</p>")
			});

			out.replace_range(marker..=marker+"<footnotes>".len(), &footnotes);
		}

		format!("<!DOCTYPE html>\n<html lang=en>\n<meta charset=\"UTF-8\">\n<link href=\"/theme.css\" rel=\"stylesheet\"/>\n{out}\n</html>")
	}

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
			// we are fancy and use em dashes, not just hyphens
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
			NodeValue::Image(NodeLink { ref url, .. }) => format!("<img src=\"{url}\" alt=\"{}\">", self.to_string(node)),
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

				ADAPTER
					.write_highlighted(&mut out, Some("shard"), literal)
					.unwrap();

				let str = String::from_utf8(out).unwrap();

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
			NodeValue::FootnoteReference(NodeFootnoteReference { ref name, .. }) => {
				if !self.footnotes.iter().any(|(n, _)| n == name) {
					self.footnotes.push((name.clone(), None));
				}

				format!("<a class=\"footnote-ref\" href=\"#footnote-{name}\"><sup>[{name}]</sup></a>")
			},
			NodeValue::FootnoteDefinition(NodeFootnoteDefinition { ref name, .. }) => {
				let desc = node.children().map(|x| self.to_string(x)).collect::<String>();

				self.footnotes.iter_mut()
					.find(|(s, _)| s == name)
					.map_or_else(
						|| println!("WARN: [{}] missing ref for footnote definition `{name}`", self.path.display()),
						|(_, d)| *d = Some(desc));

				String::new()
			},
			_ => self.to_string(node)
		}
	}
}

static SYNTAX_DIR: OnceLock<PathBuf> = OnceLock::new();
static ADAPTER: LazyLock<SyntectAdapter> = LazyLock::new(|| {
	let color = |r: u8| Color { r, g: 0, b: 0, a: 0xff };

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
