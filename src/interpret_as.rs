use std::collections::VecDeque;
use std::iter::Peekable;
use std::slice::Iter;

use crate::lex::{lex_type, TypeToken};
use crate::parse::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
	Expr,
	Boolean,
	ChurchNumeral,
	Tuple(Vec<DataType>),
}

impl TryFrom<&str> for DataType {
	type Error = ();

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let token_vec = lex_type(value);
		let mut tokens = token_vec.iter().peekable();
		let t = parse_type(&mut tokens)?;
		match tokens.next() {
			None => Ok(t),
			Some(_) => Err(()),
		}
	}
}

fn parse_type(tokens: &mut Peekable<Iter<TypeToken>>) -> Result<DataType, ()> {
	match tokens.next() {
		None => Err(()),
		Some(TypeToken::LeftSquareBracket) => Err(()),
		Some(TypeToken::RightSquareBracket) => Err(()),
		Some(TypeToken::Comma) => Err(()),
		Some(TypeToken::Expr) => Ok(DataType::Expr),
		Some(TypeToken::Bool) => Ok(DataType::Boolean),
		Some(TypeToken::Church) => Ok(DataType::ChurchNumeral),
		Some(TypeToken::Tuple) => parse_tuple_contents(tokens),
	}
}

fn parse_tuple_contents(tokens: &mut Peekable<Iter<TypeToken>>) -> Result<DataType, ()> {
	match tokens.next() {
		Some(TypeToken::LeftSquareBracket) => (),
		_ => return Err(()),
	}
	match tokens.peek() {
		Some(TypeToken::RightSquareBracket) => {
			tokens.next();
			return Ok(DataType::Tuple(Vec::new()));
		}
		_ => {}
	};
	let mut elem_types: Vec<DataType> = Vec::new();
	loop {
		let t = parse_type(tokens)?;
		elem_types.push(t);
		match tokens.next() {
			Some(TypeToken::Comma) => continue,
			Some(TypeToken::RightSquareBracket) => return Ok(DataType::Tuple(elem_types)),
			_ => return Err(()),
		}
	}
}

pub fn interpret_as(e: &Expr, dt: &DataType) -> Result<String, ()> {
	match dt {
		DataType::Expr => Ok(e.to_string()),
		DataType::Boolean => interpret_as_bool(e),
		DataType::ChurchNumeral => interpret_as_church(e),
		DataType::Tuple(elem_types) => interpret_as_tuple(e, elem_types),
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

fn interpret_as_tuple(e: &Expr, elem_types: &Vec<DataType>) -> Result<String, ()> {
	match e {
		Expr::Fun(t, body) => {
			let mut ets = elem_types.clone();
			let mut e = body;
			let mut elems: VecDeque<String> = VecDeque::new();
			loop {
				match (ets.last(), e.as_ref()) {
					(None, Expr::Var(p)) if p == t => break,
					(Some(dt), Expr::App(lhs, rhs)) => {
						let elem = interpret_as(rhs, dt)?;
						elems.push_front(elem);
						ets.truncate(ets.len() - 1);
						e = lhs;
					}
					_ => return Err(()),
				}
			}
			Ok(format!("({})", Vec::from(elems).join(", ")))
		}
		_ => Err(()),
	}
}

#[cfg(test)]
mod parse_tests {
	use crate::interpret_as::DataType;

	#[test]
	fn test_parse_expr() {
		assert_eq!(DataType::try_from("expr"), Ok(DataType::Expr));
	}

	#[test]
	fn test_parse_bool() {
		assert_eq!(DataType::try_from("bool"), Ok(DataType::Boolean));
	}

	#[test]
	fn test_parse_church() {
		assert_eq!(DataType::try_from("church"), Ok(DataType::ChurchNumeral));
	}

	#[test]
	fn test_parse_empty_tuple() {
		assert_eq!(
			DataType::try_from("tuple[]"),
			Ok(DataType::Tuple(Vec::new()))
		);
	}

	#[test]
	fn test_parse_1_tuple() {
		assert_eq!(
			DataType::try_from("tuple[expr]"),
			Ok(DataType::Tuple(vec![DataType::Expr]))
		);
	}

	#[test]
	fn test_parse_2_tuple() {
		assert_eq!(
			DataType::try_from("tuple[bool, church]"),
			Ok(DataType::Tuple(vec![
				DataType::Boolean,
				DataType::ChurchNumeral
			]))
		);
	}

	#[test]
	fn test_parse_3_tuple() {
		assert_eq!(
			DataType::try_from("tuple [ expr , bool , church ]"),
			Ok(DataType::Tuple(vec![
				DataType::Expr,
				DataType::Boolean,
				DataType::ChurchNumeral
			]))
		);
	}

	#[test]
	fn test_parse_nested_tuple() {
		assert_eq!(
			DataType::try_from("tuple[expr, tuple[bool, tuple[church]]]"),
			Ok(DataType::Tuple(vec![
				DataType::Expr,
				DataType::Tuple(vec![
					DataType::Boolean,
					DataType::Tuple(vec![DataType::ChurchNumeral])
				])
			]))
		);
	}

	#[test]
	fn test_parse_empty() {
		assert_eq!(DataType::try_from(""), Err(()));
	}

	#[test]
	fn test_parse_bool_bool() {
		assert_eq!(DataType::try_from("bool bool"), Err(()));
	}

	#[test]
	fn test_parse_tuple_unclosed_brackets() {
		assert_eq!(DataType::try_from("tuple[bool, church"), Err(()));
	}

	#[test]
	fn test_parse_extra_bracket() {
		assert_eq!(DataType::try_from("tuple[bool, bool]]"), Err(()));
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

	#[test]
	fn test_interpret_as_empty_tuple() {
		assert_eq!(
			interpret_as(&parse("\\t.t"), &DataType::Tuple(Vec::new())),
			Ok("()".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_1_tuple() {
		assert_eq!(
			interpret_as(
				&parse("\\t.t (\\t.\\f.f)"),
				&DataType::Tuple(vec![DataType::Boolean])
			),
			Ok("(false)".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_2_tuple() {
		assert_eq!(
			interpret_as(
				&parse("\\t.t (\\t.\\f.t) (\\s.\\z.s(s(z)))"),
				&DataType::Tuple(vec![DataType::Boolean, DataType::ChurchNumeral])
			),
			Ok("(true, 2)".to_owned())
		);
	}

	#[test]
	fn test_interpret_nested_tuple() {
		let e = parse("\\s.s (\\t.\\f.t) (\\s.s (\\s.s) (\\s.s (\\x.x x))) (\\s.\\z.s(s(s(z))))");
		let dt = DataType::Tuple(vec![
			DataType::Boolean,
			DataType::Tuple(vec![
				DataType::Tuple(vec![]),
				DataType::Tuple(vec![DataType::Expr]),
			]),
			DataType::ChurchNumeral,
		]);
		assert_eq!(
			interpret_as(&e, &dt),
			Ok("(true, ((), (\\x.x x)), 3)".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_tuple_wrong_element_types() {
		let e = parse("\\s.s (\\t.\\f.f) (\\s.\\z.s(s(z)))");
		let dt = DataType::Tuple(vec![DataType::Boolean, DataType::Boolean]);
		assert_eq!(interpret_as(&e, &dt), Err(()));
	}

	#[test]
	fn test_interpret_as_tuple_too_few_elements() {
		let e = parse("\\s.s (\\s.\\z.z)");
		let dt = DataType::Tuple(vec![DataType::ChurchNumeral, DataType::Boolean]);
		assert_eq!(interpret_as(&e, &dt), Err(()));
	}

	#[test]
	fn test_interpret_as_tuple_too_many_elements() {
		let e = parse("\\s.s (\\t.\\f.f) (\\t.\\f.t)");
		let dt = DataType::Tuple(vec![DataType::Boolean]);
		assert_eq!(interpret_as(&e, &dt), Err(()));
	}
}
