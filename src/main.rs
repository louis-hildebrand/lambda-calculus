use anyhow::Context;
use clap::Parser;
use lambda::*;
use std::{fs, path::PathBuf};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
	/// Lambda term to evaluate.
	#[arg(short, long)]
	term: Option<String>,
	/// File from which to read a lambda term.
	#[arg(short, long)]
	file: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	let code = match (args.term, args.file) {
		(Some(t), None) => t,
		(None, Some(f)) => {
			fs::read_to_string(&f).with_context(|| format!("unable to read {}", f.display()))?
		}
		(None, None) => anyhow::bail!("no term provided"),
		(Some(_), Some(_)) => anyhow::bail!("multiple terms provided"),
	};
	let mut stream = lex::lex(&code);
	let e = parse::parse(&mut stream);
	let output = e.to_debruijn().eval().to_named().to_string();
	println!("{output}");
	Ok(())
}
