#![no_main]

use lambda;
use lambda::parse::Expr;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: lambda::parse::Expr| {
	if is_valid(&data) {
		let s = data.to_string();
		let mut stream = lambda::lex::lex(&s).unwrap();
		stream.remove_comments();
		let e = lambda::parse::parse(&mut stream).unwrap();
		assert_eq!(*e, data);
	}
});

fn is_valid(e: &Expr) -> bool {
	match e {
		Expr::Var(s) => is_valid_identifier(s),
		Expr::Fun(x, body) => is_valid_identifier(x) && is_valid(body),
		Expr::App(e1, e2) => is_valid(e1) && is_valid(e2),
	}
}

fn is_valid_identifier(s: &str) -> bool {
	match s {
		"where" => false,
		_ if s.trim().is_empty() => false,
		_ if !s.chars().all(|c| c.is_ascii_alphanumeric()) => false,
		_ => true,
	}
}
