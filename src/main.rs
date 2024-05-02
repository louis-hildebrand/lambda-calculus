use std::fs;
use lambda::*;

fn main() {
	let file_path = "test.lam";
	let code =
		fs::read_to_string(file_path).expect(&format!("should be able to read {}", file_path));
	let mut stream = lex::lex(&code);
	let e = parse::parse(&mut stream);
	let output = e
		.to_debruijn()
		.eval()
		.to_named()
		.to_string();
	println!("{output}");
}
