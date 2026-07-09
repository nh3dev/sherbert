use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write as _;
use std::fmt::Write as _;

use toml::Value;

mod parser;
mod syntax;

const TEMPLATE_FILE: &str = "template.html";
const SYNTAX_DIR:    &str = "syntax";
const ARTICLE_DIR:   &str = "blog";
const BLACKLIST: &[&str] = &[ "files/template.html", "files/syntax/shard.sublime-syntax" ];

pub fn generate(src: &Path, dst: &Path) {
	syntax::ADAPTER.get_or_init(|| syntax::init_adapter(&src.join(SYNTAX_DIR)));

	if dst.exists() { fs::remove_dir_all(dst).unwrap(); }
	fs::create_dir(dst).unwrap();

	let template = fs::read_to_string(src.join(TEMPLATE_FILE)).unwrap();

	let mut articles = Vec::new();

	walkdir::WalkDir::new(src).into_iter()
		.filter_map(Result::ok)
		.filter(|e| !e.path().is_dir() && !BLACKLIST.contains(&e.path().to_str().unwrap()))
		.for_each(|e| {
			let path = e.path();
			let dest = dst.join(path.iter().skip(1).collect::<PathBuf>());

			match path.extension().and_then(|s| s.to_str()) {
				Some("md") => {
					let newpath = dest.with_extension("html");
					let (tags, mut body) = parser::Parser::parse(path).unwrap();
					let mut out = template.clone();

					if let Some(Value::Table(a)) = tags.get("article") {
						let Some(Value::String(title))  = a.get("title")  else { panic!("[{}]: article missing title field!", path.display())  };
						let Some(Value::String(author)) = a.get("author") else { panic!("[{}]: article missing author field!", path.display()) };
						let Some(Value::String(date))   = a.get("date")   else { panic!("[{}]: article missing date field!", path.display())   };

						body = format!("<div class=\"blog-header\"><h2>{title}</h2><p>by {author}</b><p class=\"blog-date\">{date}</p></div>\n{body}");

						articles.push((body.clone(), newpath.clone(), title.clone(), author.clone(), date.clone()));
					}

					replace_first(&mut out, "{{title}}", tags.get("title").and_then(|t| t.as_str()).unwrap_or("nh3.dev"));
					replace_first(&mut out, "{{body}}", &body);
					mkfile(&newpath, out.as_bytes()).unwrap()
				},
				_ => mkfile(&dest, &fs::read(path).unwrap()).unwrap(),
			}
		});

	articles.sort_unstable_by_key(|(_, _, _, _, date)| 
		chrono::NaiveDate::parse_from_str(date, "%d-%m-%Y").unwrap());

	let latest = articles.last().map_or_else(String::new, |(b, _, _, _, _)| b.clone());
	let article_list =  match articles.as_slice() {
		[] => String::from("<s>No articles found :(</s>"),
		_ => {
			let mut out = String::from("<div class=\"block\">");

			articles.into_iter().rev().for_each(|(_, path, title, _, date)| 
				writeln!(out, "<p><a href=\"{}\">{title} - {date}</a></p>",
					path.iter().skip(1).collect::<PathBuf>().display()).unwrap());

			write!(out, "</div>");
			out
		},
	};

	let idx_path = dst.join(ARTICLE_DIR).join("index.html");
	let mut idx = fs::read_to_string(&idx_path).unwrap();
	
	replace_first(&mut idx, "<latest>", &latest);
	replace_first(&mut idx, "<allposts>", &article_list);

	fs::write(idx_path, idx.as_bytes()).unwrap();
}

fn replace_first(s: &mut String, from: &str, to: &str) {
	if let Some(pos) = s.find(from) {
		s.replace_range(pos..pos + from.len(), to);
	}
}

fn mkfile(path: &Path, content: &[u8]) -> std::io::Result<()> {
	fs::create_dir_all(path.parent().unwrap())?;
	fs::File::create(path)?.write_all(content)
}
