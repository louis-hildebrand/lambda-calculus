use crate::parse::Expr;

pub enum DataType {
	Expr,
	Boolean,
}

impl TryFrom<&str> for DataType {
	type Error = ();

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"expr" => Ok(DataType::Expr),
			"bool" => Ok(DataType::Boolean),
			_ => Err(()),
		}
	}
}

pub fn interpret_as(e: &Expr, dt: DataType) -> Result<String, ()> {
	match dt {
		DataType::Expr => Ok(e.to_string()),
		DataType::Boolean => interpret_as_bool(e),
	}
}

fn interpret_as_bool(e: &Expr) -> Result<String, ()> {
	match e {
		Expr::Fun(a, body) => match body.as_ref() {
			Expr::Fun(b, body) => match body.as_ref() {
				Expr::Var(c) if c == a => Ok("true".to_owned()),
				Expr::Var(c) if c == b => Ok("false".to_owned()),
				_ => Err(()),
			},
			_ => Err(()),
		},
		_ => Err(()),
	}
}

#[cfg(test)]
mod interpret_as_tests {
	use crate::interpret_as::*;
	use crate::parse::Expr;

	#[test]
	fn test_interpret_id_as_expr() {
		let id = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Var("a".to_owned())),
		));
		assert_eq!(interpret_as(&id, DataType::Expr), Ok("\\a.a".to_owned()));
	}

	#[test]
	fn test_interpret_as_bool_false() {
		let f = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Fun(
				"b".to_owned(),
				Box::new(Expr::Var("b".to_owned())),
			)),
		));
		assert_eq!(interpret_as(&f, DataType::Boolean), Ok("false".to_owned()));
	}

	#[test]
	fn test_interpret_as_bool_true() {
		let f = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Fun(
				"b".to_owned(),
				Box::new(Expr::Var("a".to_owned())),
			)),
		));
		assert_eq!(interpret_as(&f, DataType::Boolean), Ok("true".to_owned()));
	}

	#[test]
	fn test_interpret_as_bool_free_var() {
		let f = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Fun(
				"b".to_owned(),
				Box::new(Expr::Var("c".to_owned())),
			)),
		));
		assert_eq!(interpret_as(&f, DataType::Boolean), Err(()));
	}

	#[test]
	fn test_interpret_as_bool_invalid_structure_1() {
		let f = Box::new(Expr::Var("a".to_owned()));
		assert_eq!(interpret_as(&f, DataType::Boolean), Err(()));
	}

	#[test]
	fn test_interpret_as_bool_invalid_structure_2() {
		let f = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Var("a".to_owned())),
		));
		assert_eq!(interpret_as(&f, DataType::Boolean), Err(()));
	}

	#[test]
	fn test_interpret_as_bool_invalid_structure_3() {
		let f = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Fun(
				"b".to_owned(),
				Box::new(Expr::App(
					Box::new(Expr::Var("a".to_owned())),
					Box::new(Expr::Var("b".to_owned())),
				)),
			)),
		));
		assert_eq!(interpret_as(&f, DataType::Boolean), Err(()));
	}
}
