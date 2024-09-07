use crate::parse::Expr;
use clap;

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum DataType {
	Expr,
	Boolean,
	ChurchNumeral,
}

impl TryFrom<&str> for DataType {
	type Error = ();

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"expr" => Ok(DataType::Expr),
			"bool" => Ok(DataType::Boolean),
			"church" => Ok(DataType::ChurchNumeral),
			_ => Err(()),
		}
	}
}

pub fn interpret_as(e: &Expr, dt: &DataType) -> Result<String, ()> {
	match dt {
		DataType::Expr => Ok(e.to_string()),
		DataType::Boolean => interpret_as_bool(e),
		DataType::ChurchNumeral => interpret_as_church(e),
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

fn interpret_as_church(e: &Expr) -> Result<String, ()> {
	match e {
		Expr::Fun(s, body) => match body.as_ref() {
			Expr::Fun(z, body) => {
				let mut n = 0;
				let mut e = body;
				loop {
					match e.as_ref() {
						Expr::Var(x) if x == z => break Ok(n.to_string()),
						Expr::App(a, b) => {
							match a.as_ref() {
								Expr::Var(x) if x == s => {}
								_ => break Err(()),
							}
							n += 1;
							e = b;
						}
						_ => break Err(()),
					}
				}
			}
			_ => Err(()),
		},
		_ => Err(()),
	}
}

#[cfg(test)]
mod interpret_as_tests {
	use crate::interpret_as::*;
	use crate::lex;
	use crate::parse;
	use crate::parse::Expr;

	fn parse(e: &str) -> Box<Expr> {
		let mut stream = lex::lex(e);
		parse::parse(&mut stream)
	}

	#[test]
	fn test_interpret_id_as_expr() {
		assert_eq!(
			interpret_as(&parse("\\x.x"), &DataType::Expr),
			Ok("\\x.x".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_bool_false() {
		assert_eq!(
			interpret_as(&parse("\\a.\\b.b"), &DataType::Boolean),
			Ok("false".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_bool_true() {
		assert_eq!(
			interpret_as(&parse("\\a.\\b.a"), &DataType::Boolean),
			Ok("true".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_bool_free_var() {
		assert_eq!(
			interpret_as(&parse("\\a.\\b.c"), &DataType::Boolean),
			Err(())
		);
	}

	#[test]
	fn test_interpret_as_bool_invalid_structure_1() {
		assert_eq!(interpret_as(&parse("a"), &DataType::Boolean), Err(()));
	}

	#[test]
	fn test_interpret_as_bool_invalid_structure_2() {
		assert_eq!(interpret_as(&parse("\\a.a"), &DataType::Boolean), Err(()));
	}

	#[test]
	fn test_interpret_as_bool_invalid_structure_3() {
		assert_eq!(
			interpret_as(&parse("\\a.\\b.a b"), &DataType::Boolean),
			Err(())
		);
	}

	#[test]
	fn test_interpret_as_church_zero() {
		assert_eq!(
			interpret_as(&parse("\\s.\\z.z"), &DataType::ChurchNumeral),
			Ok("0".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_church_one() {
		assert_eq!(
			interpret_as(&parse("\\s.\\z.s(z)"), &DataType::ChurchNumeral),
			Ok("1".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_church_two() {
		assert_eq!(
			interpret_as(&parse("\\s.\\z.s(s(z))"), &DataType::ChurchNumeral),
			Ok("2".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_church_three() {
		assert_eq!(
			interpret_as(&parse("\\s.\\z.s(s(s(z)))"), &DataType::ChurchNumeral),
			Ok("3".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_church_four() {
		assert_eq!(
			interpret_as(&parse("\\s.\\z.s(s(s(s(z))))"), &DataType::ChurchNumeral),
			Ok("4".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_church_invalid_structure_1() {
		assert_eq!(interpret_as(&parse("a"), &DataType::ChurchNumeral), Err(()));
	}

	#[test]
	fn test_interpret_as_church_invalid_structure_2() {
		assert_eq!(
			interpret_as(&parse("\\a.a"), &DataType::ChurchNumeral),
			Err(())
		);
	}

	#[test]
	fn test_interpret_as_church_invalid_structure_3() {
		assert_eq!(
			interpret_as(&parse("\\a.\\b.b a"), &DataType::ChurchNumeral),
			Err(())
		);
	}
}
