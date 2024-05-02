use crate::parse::Expr;

pub fn emit(e: &Expr) -> String {
	match e {
		Expr::Var(name) => name.clone(),
		Expr::Fun(x, body) => format!("\\{x}.{}", emit(body)),
		Expr::App(f, a) => {
			let lhs = match f.as_ref() {
				Expr::Var(_) | Expr::App(_, _) => emit(f),
				Expr::Fun(_, _) => format!("({})", emit(f)),
			};
			let rhs = match a.as_ref() {
				Expr::Var(_) => emit(a),
				Expr::Fun(_, _) | Expr::App(_, _) => format!("({})", emit(a)),
			};
			format!("{lhs} {rhs}")
		}
	}
}

#[cfg(test)]
mod emit_tests {
	use crate::emit::emit;
	use crate::parse::Expr;

	#[test]
	fn emit_var() -> () {
		let e = Expr::Var("x".to_owned());
		assert_eq!("x", emit(&e));
	}

	#[test]
	fn emit_identity() -> () {
		let e = Expr::Fun("z".to_owned(), Box::new(Expr::Var("z".to_owned())));
		assert_eq!("\\z.z", emit(&e));
	}

	#[test]
	fn emit_app_var_var() -> () {
		let e = Expr::App(
			Box::new(Expr::Var("s".to_owned())),
			Box::new(Expr::Var("z".to_owned())),
		);
		assert_eq!("s z", emit(&e));
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
		assert_eq!("f (\\x.x)", emit(&e));
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
		assert_eq!("s (s z)", emit(&e));
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
		assert_eq!("(\\x.x) y", emit(&e));
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
		assert_eq!("(\\x.x) (\\y.y)", emit(&e));
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
		assert_eq!("(\\x.x) (a b)", emit(&e));
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
		assert_eq!("x y z", emit(&e));
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
		assert_eq!("x y (\\z.z)", emit(&e));
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
		assert_eq!("x y (z w)", emit(&e));
	}
}
