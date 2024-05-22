use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command, fs};

#[test]
pub fn test_examples() -> Result<(), Box<dyn Error>> {
	let files = fs::read_dir("./examples")?;
	for ff in files {
		let f = ff.unwrap();
		let contents = fs::read_to_string(f.path()).unwrap();
		let result_line = contents.split("\n").filter(|ln| ln.starts_with("{ RESULT: ")).nth(0);
		let result = match result_line {
			None => panic!("No result found in {:?}", f),
			Some(s) => s.strip_prefix("{ RESULT: ").unwrap().strip_suffix(" }").unwrap(),
		};
		
		let mut cmd = Command::cargo_bin("lambda")?;
		cmd.arg("-f").arg(f.path().to_str().unwrap());
		let p = predicate::eq(format!("{}\n", result));
		cmd.assert().success().stdout(p);

		let mut cmd = Command::cargo_bin("lambda")?;
		cmd.arg("-t").arg(&contents);
		let p = predicate::eq(format!("{}\n", result));
		cmd.assert().success().stdout(p);
	}
	Ok(())
}
