use crate::interpret_as::{interpret_as, DataType};
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
	/// What datatype to interpret the result as.
	#[arg(short, long, default_value = "expr")]
	interpret_as: DataType,
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
	let evaluated = e.to_debruijn().eval().to_named();
	let output = match interpret_as(&evaluated, &args.interpret_as) {
		Ok(s) => s,
		Err(()) => format!("Failed to interpret result as {:?}.", &args.interpret_as),
	};
	println!("{output}");
	Ok(())
}
