pub mod debruijn;
pub mod emit;
pub mod eval;
pub mod interpret_as;
pub mod lex;
pub mod parse;

use interpret_as::interpret_as;
use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
	// When the `console_error_panic_hook` feature is enabled, we can call the
	// `set_panic_hook` function at least once during initialization, and then
	// we will get better error messages if our code ever panics.
	//
	// For more details see
	// https://github.com/rustwasm/console_error_panic_hook#readme
	#[cfg(feature = "console_error_panic_hook")]
	console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn eval_lambda(src: &str) -> String {
	set_panic_hook();
	// TODO: handle errors gracefully
	let mut stream = lex::lex(src);
	stream.remove_comments();
	let e = parse::parse(&mut stream);
	let evaluated = e.to_debruijn().eval().to_named();
	stream = lex::lex(src);
	let datatype = parse::find_type_annotation(&mut stream).unwrap_or("expr".to_owned());
	let out = match datatype.as_str().try_into() {
		Ok(dt) => interpret_as(&evaluated, &dt),
		Err(()) => Err(()),
	};
	match out {
		Ok(s) => s,
		Err(()) => format!("Failed to interpret result as {datatype}."),
	}
}
