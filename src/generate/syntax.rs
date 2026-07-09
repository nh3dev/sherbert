use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use syntect::highlighting::{ThemeSet, Theme, ThemeSettings, ThemeItem, ScopeSelector, ScopeSelectors, StyleModifier, Color, FontStyle};
use syntect::parsing::{SyntaxSetBuilder, Scope, ScopeStack};

use std::sync::OnceLock;
use std::path::Path;

pub static ADAPTER: OnceLock<SyntectAdapter> = OnceLock::new();
pub fn init_adapter(syntax_dir: &Path) -> SyntectAdapter {
	let color = |r: u8| Color { r, g: 0, b: 0, a: 0xff };

	let item = |path: &str, id: u8, style: Option<FontStyle>| ThemeItem {
		scope: ScopeSelectors { selectors: vec![ScopeSelector { path: ScopeStack::from_vec(vec![Scope::new(path).unwrap()]), excludes: Vec::new()}]},
		style: StyleModifier {
			foreground: Some(color(id)),
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
	set.add_from_folder(syntax_dir, true).unwrap();

	SyntectAdapterBuilder::new()
		.theme_set(ThemeSet {
			themes: {
				let mut themes = std::collections::BTreeMap::new();
				themes.insert("theme".to_string(), theme);
				themes
			}
		})
		.theme("theme")
		.syntax_set(set.build())
		.build()
}
