use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, fs, path::PathBuf, process::Command};

fn test_eval_from_file(p: &PathBuf, result: &str, dt: &str) -> Result<(), Box<dyn Error>> {
	let mut cmd = Command::cargo_bin("lambda")?;
	cmd.arg("-f")
		.arg(p.to_str().unwrap())
		.arg("--interpret-as")
		.arg(dt);
	let p = predicate::eq(format!("{}\n", result));
	cmd.assert().success().stdout(p);
	Ok(())
}

fn test_eval_term(e: &str, result: &str, dt: &str) -> Result<(), Box<dyn Error>> {
	let mut cmd = Command::cargo_bin("lambda")?;
	cmd.arg("-t").arg(e).arg("--interpret-as").arg(dt);
	let p = predicate::eq(format!("{}\n", result));
	cmd.assert().success().stdout(p);
	Ok(())
}

#[test]
pub fn test_examples() -> Result<(), Box<dyn Error>> {
	let files = fs::read_dir("./examples")?;
	for ff in files {
		let f = ff.unwrap();
		let contents = fs::read_to_string(f.path()).unwrap();

		let result_line = contents
			.split("\n")
			.filter(|ln| ln.starts_with("{ RESULT: "))
			.nth(0);
		let result = match result_line {
			None => panic!("No result found in {:?}", f),
			Some(s) => s
				.strip_prefix("{ RESULT: ")
				.unwrap()
				.strip_suffix(" }")
				.unwrap()
				.trim(),
		};
		test_eval_term(&contents, result, "expr")?;
		test_eval_from_file(&f.path(), result, "expr")?;

		let bool_result_line = contents
			.split("\n")
			.filter(|ln| ln.starts_with("{ RESULT-BOOL: "))
			.nth(0);
		match bool_result_line {
			None => {}
			Some(s) => {
				let bool_result = s
					.strip_prefix("{ RESULT-BOOL: ")
					.unwrap()
					.strip_suffix(" }")
					.unwrap()
					.trim();
				test_eval_term(&contents, bool_result, "boolean")?;
				test_eval_from_file(&f.path(), bool_result, "boolean")?;
			}
		}

		let church_result_line = contents
			.split("\n")
			.filter(|ln| ln.starts_with("{ RESULT-CHURCH-NUM: "))
			.nth(0);
		match church_result_line {
			None => {}
			Some(s) => {
				let church_result = s
					.strip_prefix("{ RESULT-CHURCH-NUM: ")
					.unwrap()
					.strip_suffix(" }")
					.unwrap()
					.trim();
				test_eval_term(&contents, church_result, "church-numeral")?;
				test_eval_from_file(&f.path(), church_result, "church-numeral")?;
			}
		}

		// Check that there are no lines starting with "{ RESULT:" other than
		// those already handled
		for ln in contents.split("\n").filter(|ln| ln.starts_with("{")) {
			match result_line {
				Some(s) if s == ln => continue,
				_ => {}
			};
			match bool_result_line {
				Some(s) if s == ln => continue,
				_ => {}
			};
			match church_result_line {
				Some(s) if s == ln => continue,
				_ => {}
			};
			panic!("Unhandled result line '{ln}'.");
		}
	}
	Ok(())
}
