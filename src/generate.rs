use comrak::{parse_document, Arena, Options};
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::nodes::{AstNode, NodeCode, NodeCodeBlock, NodeValue, NodeHtmlBlock, NodeHeading, NodeLink, NodeList};
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};

use syntect::highlighting::{ThemeSet, Theme, ThemeSettings, ThemeItem, ScopeSelector, ScopeSelectors, StyleModifier, Color, FontStyle};
use syntect::parsing::{SyntaxSetBuilder, Scope, ScopeStack};

use regex::{Regex, Captures};

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::{LazyLock, OnceLock};

use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write as _;
use std::fmt::Write as _;

// workaround since syntax is loaded in a static and I cant be bothered to move it
static SYNTAX_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn generate(src: &Path, dst: &Path, syntax: PathBuf) {
	static BLOG_ENTRY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<blog-header "(.*)", "(.*)", "(.*)">"#).unwrap());
	static INCLUDE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<include "(.*)">"#).unwrap());

	SYNTAX_DIR.get_or_init(|| syntax);

	let resolve_macros = |mut input: String| {
		let f  = |c: &Captures| fs::read_to_string(src.join(&c[1])).unwrap();
		if let Cow::Owned(s) = INCLUDE_RE.replace_all(&input, f) { input = s; }

		let f  = |c: &Captures| format!(r#"<div class="blog-header"><h2>{}</h2><p>by {}</b><p class="blog-date">{}</p></div>"#, &c[1], &c[2], &c[3]);
		if let Cow::Owned(s) = BLOG_ENTRY_RE.replace_all(&input, f) { input = s; }

		input
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
		fs::remove_dir_all(dst).unwrap(); 
	}

	fs::create_dir(dst).unwrap();

	let blog_dir = src.join("blog");

	walkdir::WalkDir::new(src).into_iter()
		.filter_map(Result::ok).filter(|e| !e.path().is_dir()).for_each(|e| {
			let path = e.path();
			let dest = dst.join(path.iter().skip(1).collect::<PathBuf>());

			if path.starts_with(&blog_dir) { return; }

			match path.extension().and_then(|s| s.to_str()) {
				Some("md")   => mkfile(&dest.with_extension("html")).write_all(resolve_macros(into_html(path)).as_bytes()),
				Some("html") => mkfile(&dest).write_all(resolve_macros(fs::read_to_string(path).unwrap()).as_bytes()),
				_            => mkfile(&dest).write_all(&fs::read(path).unwrap()),
			}.unwrap();
		});

	let mut posts = std::fs::read_dir(&blog_dir).unwrap().filter_map(|e| {
			let path = e.ok()?.path();

			if path.file_name()? == "index.md" { return None; }

			let content = std::fs::read_to_string(&path).ok()?;
			let c = BLOG_ENTRY_RE.captures(&content)?;
			let (name, auth, date) = (c[1].to_string(), c[2].to_string(), c[3].to_string());

			let content = into_html(&path);
			let newpath = dst.join(path.iter().skip(1).collect::<PathBuf>()).with_extension("html");
			mkfile(&newpath).write_all(resolve_macros(content.clone()).as_bytes());

			Some((content, name, auth, date, newpath))
		}).collect::<Vec<_>>();

	posts.sort_unstable_by_key(|(_, _, _, s, _)| {
		let mut it = s.split("-");
		(it.next().unwrap().parse::<u32>().unwrap(),
			it.next().unwrap().parse::<u32>().unwrap(),
			it.next().unwrap().parse::<u32>().unwrap())
	});

	let latest = posts.first().map_or_else(
		||  String::from("<s>No posts found :(</s>"),
		|p| INCLUDE_RE.replace_all(&p.0, |_: &Captures| "").into_owned());

	let blog_list = {
		let mut out = String::from("<div class=\"block\">");

		posts.into_iter().for_each(|(_, name, _, date, path)| 
			writeln!(out, "<p><a href=\"{}\">{name} - {date}</a></p>",
				path.iter().skip(1).collect::<PathBuf>().display()).unwrap());

		write!(out, "</div>");
		out
	};

	let blog_idx = into_html(&blog_dir.join("index.md"))
		.replacen("<latest>", &latest, 1)
		.replacen("<allposts>", &blog_list, 1);

	mkfile(&dbg!(dst.join("blog/index.html"))).write_all(resolve_macros(blog_idx).as_bytes());
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
						true => writeln!(out, "<th>{}</th>", to_string(cell)),
						_    => writeln!(out, "<td>{}</td>", to_string(cell)),
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
