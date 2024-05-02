use std::fs;
use lambda::*;

fn main() {
	let file_path = "test.lam";
	let code =
		fs::read_to_string(file_path).expect(&format!("should be able to read {}", file_path));
	let mut stream = lex::lex(&code);
	let e = parse::parse(&mut stream);
	let de = debruijn::to_debruijn(&e);
	let eval_de = eval::eval(&de);
	let eval_e = debruijn::to_named(&eval_de);
	let output = emit::emit(&eval_e);
	println!("{output}");
}
