use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write as _;
use std::cell::UnsafeCell;

use toml::{Value, Table};

mod parser;
mod syntax;

const SYNTAX_DIR:   &str = "syntax";
const ARTICLE_DIR:  &str = "blog";
const TEMPLATE_DIR: &str = "templates";

#[derive(Debug)]
struct Template {
	body:    String,
	inherit: Option<String>,
	tags:    Table,
}

struct Engine {
	templates: Vec<(String, Template)>,
}

impl Engine {
	fn new(src: &Path) -> Self {
		let templates = fs::read_dir(src).unwrap()
			.filter_map(Result::ok)
			.filter(|e| e.path().is_file())
			.map(|e| {
				let path = e.path();
				let (mut tags, body) = split_header(fs::read_to_string(&path).unwrap())
					.unwrap_or_else(|e| panic!("{e}"));

				let name = path.file_stem().unwrap().to_str().unwrap().to_string();
				let inherit = tags.remove("inherits").map(|v| v.as_str().unwrap().to_owned());

				(name, Template { body, inherit, tags })
			})
			.collect::<Vec<_>>();

		Self { templates }
	}

	pub fn generate(src: &Path, dst: &Path) {
		syntax::ADAPTER.get_or_init(|| syntax::init_adapter(&src.join(SYNTAX_DIR)));

		if dst.exists() { fs::remove_dir_all(dst).unwrap(); }
		fs::create_dir(dst).unwrap();

		let this = Self::new(&src.join(TEMPLATE_DIR));
		let mut articles = Vec::new();

		let files = walkdir::WalkDir::new(src).into_iter()
			.filter_map(Result::ok)
			.filter(|e| !e.path().is_dir() 
				&& e.path().parent().and_then(|p| p.to_str()) != Some(TEMPLATE_DIR)
				&& e.path().parent().and_then(|p| p.to_str()) != Some(SYNTAX_DIR))
			.filter_map(|e| {
				let path = e.path();
				let dest = dst.join(path.iter().skip(1).collect::<PathBuf>());

				if path.extension().is_some_and(|s| s.to_str().unwrap() != "md") {
					mkfile(&dest, fs::read(path).unwrap()).unwrap();
					return None;
				}

				let newpath = dest.with_extension("html");
				let (mut tags, body) = parser::Parser::parse(path).unwrap_or_else(|e| panic!("{e}"));

				if let Some(Value::Table(a)) = tags.get("article") {
					let a = a.clone();
					let Some(Value::String(title))  = a.get("title")  else { panic!("[{}]: article missing title field!", path.display())  };
					let Some(Value::String(author)) = a.get("author") else { panic!("[{}]: article missing author field!", path.display()) };
					let Some(Value::String(date))   = a.get("date")   else { panic!("[{}]: article missing date field!", path.display())   };

					tags.insert(String::from("template"), Value::String(String::from("article")));

					let og = OwnCell::new(format!("<meta property=\"og:title\" content=\"{title}\"/>"));

					tags.insert_or_edit("opengraph",
						||  Value::Array(vec![Value::String(og.take())]),
						|v| v.as_array_mut().unwrap().push(Value::String(og.take())));

					articles.push((newpath.clone(), title.clone(), author.clone(), date.clone()));
				}

				if let Some(Value::String(summary)) = tags.get("summary") {
					let summary = OwnCell::new(format!("<meta property=\"og:description\" content=\"{summary}\"/>"));
					tags.insert_or_edit("opengraph",
						||  Value::Array(vec![Value::String(summary.take())]),
						|v| v.as_array_mut().unwrap().push(Value::String(summary.take())));
				}

				Some((tags, body, path.to_owned(), newpath))
			}).collect::<Vec<_>>();

		articles.sort_unstable_by_key(|(_, _, _, date)| 
			chrono::NaiveDate::parse_from_str(date, "%d-%m-%Y").unwrap());

		let article_paths = articles.iter().map(|(p, _, _, _)| p.clone()).collect::<Vec<_>>();
		let article_index = articles.iter().rev().map(|(path, title, _, date)|
			format!("<p><a href=\"/{}\">{title}<span class=\"tag-date\">{date}</span></a></p>",
				path.iter().skip(1).collect::<PathBuf>().display()))
			.map(Value::String)
			.collect::<Vec<_>>();

		files.into_iter().for_each(|(mut tags, body, path, newpath)| {
			let template = match tags.get("template") {
				Some(Value::String(name)) => name.clone(),
				Some(_) => panic!("[{}]: template must be type `string`", path.display()),
				None => String::from("default"),
			};

			if article_paths.contains(&newpath) {
				tags.insert(String::from("article-index"), Value::Array(article_index.clone()));
			}

			let (tags, rest) = this.resolve_template((tags, &body), &template);
			let body = tags.replace_tags_from(&rest);

			if article_paths.last() == Some(&newpath) {
				mkfile(&dst.join(ARTICLE_DIR).join("index.html"), body.as_bytes()).unwrap();
			}

			mkfile(&newpath, body.as_bytes()).unwrap()
		});
	}

	fn resolve_template(&self, (tags, body): (Table, &str), inherit: &str) -> (Table, String) {
		let (_, t) = self.templates.iter().find(|(n, _)| n == inherit).expect("invalid template");

		let (mut t, mut b) = match t.inherit {
			Some(ref i) => self.resolve_template((t.tags.clone(), &t.body), i),
			None => (t.tags.clone(), t.body.clone()),
		};

		replace_first(&mut b, "{{body}}", body);
		t.merge_into(tags);

		(t, b)
	}
}

pub fn split_header(src: String) -> Result<(Table, String), toml::de::Error> {
	Ok(match src.split_once("---\n") {
		Some((pre, body)) => (pre.parse::<Table>()?, body.to_string()),
		None => (Table::new(), src),
	})
}

fn mkfile(path: &Path, content: impl AsRef<[u8]>) -> std::io::Result<()> {
	fs::create_dir_all(path.parent().unwrap())?;
	fs::File::create(path)?.write_all(content.as_ref())
}

fn replace_first(s: &mut String, from: impl AsRef<str>, to: &str) {
	if let Some(pos) = s.find(from.as_ref()) {
		s.replace_range(pos..pos + from.as_ref().len(), to);
	}
}

fn display_value(v: &Value) -> String {
	match v {
		Value::String(s)  => s.clone(),
		Value::Boolean(b) => format!("{b:?}"),
		Value::Array(a)   => a.iter().map(|v| display_value(v) + "\n").collect::<String>(),
		_ => v.to_string(),
	}
}


trait TableExt {
	fn replace_tags_from(&self, rest: &str) -> String;
	fn merge_into(&mut self, other: Self);
	fn get_from_expr<'a>(&'a self, expr: &str) -> Option<&'a Value>;
	fn insert_or_edit(&mut self, k: &str, insert: impl FnOnce() -> Value, edit: impl FnOnce(&mut Value));
}

impl TableExt for Table {
	fn replace_tags_from(&self, mut rest: &str) -> String {
		let mut out = String::with_capacity(rest.len());

		while let Some(start) = rest.find("{{") {
			out.push_str(&rest[..start]);
			rest = &rest[start + 2..];

			match rest.find("}}") {
				Some(end) => {
					let Some(v) = self.get_from_expr(&rest[..end]) else {
						panic!("undefined value {}", &rest[..end]);
					};

					out.push_str(&display_value(v));
					rest = &rest[end + 2..];
				},
				None => {
					out.push_str("{{");
					break;
				},
			}
		}

		out.push_str(rest);
		out
	}

	fn merge_into(&mut self, other: Self) {
		other.into_iter().for_each(|(k, v)|
			match self.get_mut(&k) {
				None => { self.insert(k, v); },
				Some(Value::String(_))   if let Value::String(_)   = v => { self.insert(k, v); },
				Some(Value::Float(_))    if let Value::Float(_)    = v => { self.insert(k, v); },
				Some(Value::Boolean(_))  if let Value::Boolean(_)  = v => { self.insert(k, v); },
				Some(Value::Integer(_))  if let Value::Integer(_)  = v => { self.insert(k, v); },
				Some(Value::Datetime(_)) if let Value::Datetime(_) = v => { self.insert(k, v); },
				Some(Value::Table(t))    if let Value::Table(o)    = v => t.merge_into(o),
				Some(Value::Array(a1))   if let Value::Array(a2)   = v => a1.extend(a2),
				_ => panic!("incompatible types in table merge"),
			});
	}

	fn get_from_expr<'a>(mut self: &'a Self, expr: &str) -> Option<&'a Value> {
		if expr.is_empty() { panic!(); }

		let parts = expr.split(".").collect::<Vec<_>>();
		let len = parts.len();
		let mut parts = parts.into_iter().enumerate();

		loop {
			let Some((i, p)) = parts.next() else { break None; };
			match self.get(p) {
				Some(Value::Table(t)) => self = t,
				Some(v) if i + 1 == len => break Some(v),
				Some(_) => panic!("attempt to index a non `Table` value"),
				None => break None,
			}
		}
	}

	fn insert_or_edit(&mut self, k: &str, insert: impl FnOnce() -> Value, edit: impl FnOnce(&mut Value)) {
		match self.get_mut(k) {
			Some(v) => edit(v),
			None => { self.insert(k.to_owned(), insert()); },
		}
	}
}

struct OwnCell<T>(UnsafeCell<Option<T>>);

impl<T> OwnCell<T> {
	pub fn new(v: T) -> Self {
		Self(UnsafeCell::new(Some(v)))
	}

	pub fn take(&self) -> T {
		unsafe { &mut *self.0.get() }.take().expect("OwnCell already taken")
	}
}


pub fn generate(src: &Path, dst: &Path) {
	Engine::generate(src, dst);
}
