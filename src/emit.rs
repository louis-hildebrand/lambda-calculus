use std::fmt::Display;

use crate::parse::Expr;

impl Display for Expr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Expr::Var(name) => name.clone(),
			Expr::Fun(x, body) => format!("\\{x}.{}", body.to_string()),
			Expr::App(f, a) => {
				let lhs = match f.as_ref() {
					Expr::Var(_) | Expr::App(_, _) => f.to_string(),
					Expr::Fun(_, _) => format!("({})", f.to_string()),
				};
				let rhs = match a.as_ref() {
					Expr::Var(_) => a.to_string(),
					Expr::Fun(_, _) | Expr::App(_, _) => format!("({})", a.to_string()),
				};
				format!("{lhs} {rhs}")
			}
		};
		f.write_str(&s)
	}
}

#[cfg(test)]
mod emit_tests {
	use crate::parse::Expr;

	#[test]
	fn emit_var() -> () {
		let e = Expr::Var("x".to_owned());
		assert_eq!("x", e.to_string());
	}

	#[test]
	fn emit_identity() -> () {
		let e = Expr::Fun("z".to_owned(), Box::new(Expr::Var("z".to_owned())));
		assert_eq!("\\z.z", e.to_string());
	}

	#[test]
	fn emit_app_var_var() -> () {
		let e = Expr::App(
			Box::new(Expr::Var("s".to_owned())),
			Box::new(Expr::Var("z".to_owned())),
		);
		assert_eq!("s z", e.to_string());
	}

	#[test]
	fn emit_app_var_fun() -> () {
		let e = Expr::App(
			Box::new(Expr::Var("f".to_owned())),
			Box::new(Expr::Fun(
				"x".to_owned(),
				Box::new(Expr::Var("x".to_owned())),
			)),
		);
		assert_eq!("f (\\x.x)", e.to_string());
	}

	#[test]
	fn emit_app_var_app() -> () {
		let e = Expr::App(
			Box::new(Expr::Var("s".to_owned())),
			Box::new(Expr::App(
				Box::new(Expr::Var("s".to_owned())),
				Box::new(Expr::Var("z".to_owned())),
			)),
		);
		assert_eq!("s (s z)", e.to_string());
	}

	#[test]
	fn emit_app_fun_var() -> () {
		let e = Expr::App(
			Box::new(Expr::Fun(
				"x".to_owned(),
				Box::new(Expr::Var("x".to_owned())),
			)),
			Box::new(Expr::Var("y".to_owned())),
		);
		assert_eq!("(\\x.x) y", e.to_string());
	}

	#[test]
	fn emit_app_fun_fun() -> () {
		let e = Expr::App(
			Box::new(Expr::Fun(
				"x".to_owned(),
				Box::new(Expr::Var("x".to_owned())),
			)),
			Box::new(Expr::Fun(
				"y".to_owned(),
				Box::new(Expr::Var("y".to_owned())),
			)),
		);
		assert_eq!("(\\x.x) (\\y.y)", e.to_string());
	}

	#[test]
	fn emit_app_fun_app() -> () {
		let e = Expr::App(
			Box::new(Expr::Fun(
				"x".to_owned(),
				Box::new(Expr::Var("x".to_owned())),
			)),
			Box::new(Expr::App(
				Box::new(Expr::Var("a".to_owned())),
				Box::new(Expr::Var("b".to_owned())),
			)),
		);
		assert_eq!("(\\x.x) (a b)", e.to_string());
	}

	#[test]
	fn emit_app_app_var() -> () {
		let e = Expr::App(
			Box::new(Expr::App(
				Box::new(Expr::Var("x".to_owned())),
				Box::new(Expr::Var("y".to_owned())),
			)),
			Box::new(Expr::Var("z".to_owned())),
		);
		assert_eq!("x y z", e.to_string());
	}

	#[test]
	fn emit_app_app_fun() -> () {
		let e = Expr::App(
			Box::new(Expr::App(
				Box::new(Expr::Var("x".to_owned())),
				Box::new(Expr::Var("y".to_owned())),
			)),
			Box::new(Expr::Fun(
				"z".to_owned(),
				Box::new(Expr::Var("z".to_owned())),
			)),
		);
		assert_eq!("x y (\\z.z)", e.to_string());
	}

	#[test]
	fn emit_app_app_app() -> () {
		let e = Expr::App(
			Box::new(Expr::App(
				Box::new(Expr::Var("x".to_owned())),
				Box::new(Expr::Var("y".to_owned())),
			)),
			Box::new(Expr::App(
				Box::new(Expr::Var("z".to_owned())),
				Box::new(Expr::Var("w".to_owned())),
			)),
		);
		assert_eq!("x y (z w)", e.to_string());
	}
}
