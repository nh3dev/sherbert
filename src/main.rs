#![allow(unused_must_use)]

use std::path::PathBuf;

use clap::Parser;

mod host;
mod generate;

#[derive(Parser)]
#[command(version, about)]
struct Args {
	#[clap(subcommand)]
	command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
	Host {
		dir:  PathBuf,
		#[arg(short, long, default_value = "127.0.0.1:8080")]
		addr: String,
		#[arg(short, long)]
		theme: String,
	},
	Generate {
		source: PathBuf,
		output: PathBuf,
		syntax: PathBuf,
	},
}

fn main() {
	match Args::parse().command {
		Command::Host { dir, addr, theme } => host::host(dir, &addr, theme),
		Command::Generate { source, output, syntax } => generate::generate(&source, &output, syntax),
	}
}
