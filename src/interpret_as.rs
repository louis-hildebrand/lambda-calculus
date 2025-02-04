use std::collections::VecDeque;
use std::iter::Peekable;
use std::slice::Iter;

use crate::error::Error;
use crate::lex::{lex_type, TypeToken};
use crate::parse::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
	Expr,
	Boolean,
	ChurchNumeral,
	Tuple(Vec<DataType>),
	List(Box<DataType>),
}

impl TryFrom<&str> for DataType {
	type Error = crate::error::Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let token_vec = lex_type(value)?;
		let mut tokens = token_vec.iter().peekable();
		let t = parse_type(&mut tokens)?;
		match tokens.next() {
			None => Ok(t),
			Some(t) => Err(Error::MalformedType(format!(
				"unexpected token after end of type: \"{t}\""
			))),
		}
	}
}

fn parse_type(tokens: &mut Peekable<Iter<TypeToken>>) -> Result<DataType, Error> {
	match tokens.next() {
		None => Err(Error::MalformedType("unexpected end of type".to_owned())),
		Some(TypeToken::LeftSquareBracket) => Err(Error::MalformedType(
			"unexpected character: \"[\"".to_owned(),
		)),
		Some(TypeToken::RightSquareBracket) => Err(Error::MalformedType(
			"unexpected character: \"]\"".to_owned(),
		)),
		Some(TypeToken::Comma) => Err(Error::MalformedType(
			"unexpected character: \",\"".to_owned(),
		)),
		Some(TypeToken::Expr) => Ok(DataType::Expr),
		Some(TypeToken::Bool) => Ok(DataType::Boolean),
		Some(TypeToken::Church) => Ok(DataType::ChurchNumeral),
		Some(TypeToken::Tuple) => parse_tuple_contents(tokens),
		Some(TypeToken::List) => parse_list_contents(tokens),
	}
}

fn parse_tuple_contents(tokens: &mut Peekable<Iter<TypeToken>>) -> Result<DataType, Error> {
	match tokens.next() {
		Some(TypeToken::LeftSquareBracket) => (),
		Some(t) => {
			return Err(Error::MalformedType(format!(
				"expected next token in tuple type to be \"[\" but found \"{t}\""
			)))
		}
		None => {
			return Err(Error::MalformedType(
				"expected next token in tuple type to be \"[\" but found nothing".to_owned(),
			))
		}
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
			Some(t) => {
				return Err(Error::MalformedType(format!(
					"expected next token in tuple type to be \",\" or \"]\" but found \"{t}\""
				)))
			}
			None => {
				return Err(Error::MalformedType(format!(
					"expected next token in tuple type to be \",\" or \"]\" but found nothing"
				)))
			}
		}
	}
}

fn parse_list_contents(tokens: &mut Peekable<Iter<TypeToken>>) -> Result<DataType, Error> {
	match tokens.next() {
		Some(TypeToken::LeftSquareBracket) => (),
		Some(t) => {
			return Err(Error::MalformedType(format!(
				"expected next token in list type to be \"[\" but found \"{t}\""
			)))
		}
		None => {
			return Err(Error::MalformedType(
				"expected next token in list type to be \"[\" but found nothing".to_owned(),
			))
		}
	};
	let t = parse_type(tokens)?;
	match tokens.next() {
		Some(TypeToken::RightSquareBracket) => (),
		Some(t) => {
			return Err(Error::MalformedType(format!(
				"expected next token in list type to be \"]\" but found \"{t}\""
			)))
		}
		None => {
			return Err(Error::MalformedType(
				"expected next token in list type to be \"]\" but found nothing".to_owned(),
			))
		}
	};
	Ok(DataType::List(Box::new(t)))
}

pub fn interpret_as(e: &Expr, dt: &DataType) -> Result<String, ()> {
	match dt {
		DataType::Expr => Ok(e.to_string()),
		DataType::Boolean => interpret_as_bool(e),
		DataType::ChurchNumeral => interpret_as_church(e),
		DataType::Tuple(elem_types) => interpret_as_tuple(e, elem_types),
		DataType::List(t) => interpret_as_list(e, t),
	}
}

fn interpret_as_bool(e: &Expr) -> Result<String, ()> {
	match e {
		Expr::Fun(t, body) => match body.as_ref() {
			Expr::Fun(f, body) => match body.as_ref() {
				Expr::Var(c) if c == t => Ok("true".to_owned()),
				Expr::Var(c) if c == f => Ok("false".to_owned()),
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
		Expr::Fun(s, body) => {
			let mut ets = elem_types.clone();
			let mut e = body;
			let mut elems: VecDeque<String> = VecDeque::new();
			loop {
				match (ets.last(), e.as_ref()) {
					(None, Expr::Var(p)) if p == s => break,
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

fn interpret_as_list(e: &Expr, dt: &DataType) -> Result<String, ()> {
	Ok(format!("[{}]", interpret_as_list_without_brackets(e, dt)?))
}

fn interpret_as_list_without_brackets(e: &Expr, dt: &DataType) -> Result<String, ()> {
	match e {
		_ if is_nil(e) => Ok("".to_owned()),
		Expr::Fun(s, body) => match body.as_ref() {
			Expr::App(lhs, tail) => match lhs.as_ref() {
				Expr::App(p, head) => match p.as_ref() {
					Expr::Var(p) if p == s => {
						let head_str = interpret_as(head, dt)?;
						let tail_str = interpret_as_list_without_brackets(tail, dt)?;
						if tail_str.trim().is_empty() {
							Ok(head_str)
						} else {
							Ok(format!("{head_str}, {tail_str}"))
						}
					}
					_ => Err(()),
				},
				_ => Err(()),
			},
			_ => Err(()),
		},
		_ => Err(()),
	}
}

fn is_nil(e: &Expr) -> bool {
	match e {
		Expr::Fun(_, body) => match body.as_ref() {
			// Check if body is true
			Expr::Fun(t, body) => match body.as_ref() {
				Expr::Fun(_, body) => match body.as_ref() {
					Expr::Var(c) if c == t => true,
					_ => false,
				},
				_ => false,
			},
			_ => false,
		},
		_ => false,
	}
}

#[cfg(test)]
mod parse_tests {
	use crate::{error::Error, interpret_as::DataType};

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
	fn test_parse_list_bool() {
		assert_eq!(
			DataType::try_from("list[bool]"),
			Ok(DataType::List(Box::new(DataType::Boolean)))
		);
	}

	#[test]
	fn test_parse_list_church() {
		assert_eq!(
			DataType::try_from("list[church]"),
			Ok(DataType::List(Box::new(DataType::ChurchNumeral)))
		);
	}

	#[test]
	fn test_parse_empty() {
		assert_eq!(
			DataType::try_from(""),
			Err(Error::MalformedType("unexpected end of type".to_owned()))
		);
	}

	#[test]
	fn test_parse_bool_bool() {
		assert_eq!(
			DataType::try_from("bool bool"),
			Err(Error::MalformedType(
				"unexpected token after end of type: \"bool\"".to_owned()
			))
		);
	}

	#[test]
	fn test_parse_tuple_missing_types() {
		assert_eq!(
			DataType::try_from("tuple"),
			Err(Error::MalformedType(
				"expected next token in tuple type to be \"[\" but found nothing".to_owned()
			))
		)
	}

	#[test]
	fn test_parse_tuple_unclosed_brackets() {
		assert_eq!(
			DataType::try_from("tuple[bool, church"),
			Err(Error::MalformedType(
				"expected next token in tuple type to be \",\" or \"]\" but found nothing"
					.to_owned()
			))
		);
	}

	#[test]
	fn test_parse_tuple_missing_comma() {
		assert_eq!(
			DataType::try_from("tuple[bool church]"),
			Err(Error::MalformedType(
				"expected next token in tuple type to be \",\" or \"]\" but found \"church\""
					.to_owned()
			))
		);
	}

	#[test]
	fn test_parse_extra_bracket() {
		assert_eq!(
			DataType::try_from("tuple[bool, bool]]"),
			Err(Error::MalformedType(
				"unexpected token after end of type: \"]\"".to_owned()
			))
		);
	}
}

#[cfg(test)]
mod interpret_as_tests {
	use crate::interpret_as::*;
	use crate::lex;
	use crate::parse;
	use crate::parse::Expr;

	fn parse(e: &str) -> Box<Expr> {
		let mut stream = lex::lex(e).unwrap();
		parse::parse(&mut stream).unwrap()
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

	#[test]
	fn test_interpret_as_list_nil() {
		assert_eq!(
			interpret_as(
				&parse("\\_.T where T = \\t.\\f.t"),
				&DataType::List(Box::new(DataType::Expr))
			),
			Ok("[]".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_list_bool() {
		let dt = DataType::List(Box::new(DataType::Boolean));
		let t = "\\t.\\f.t";
		let f = "\\t.\\f.f";
		let mut e = format!("\\_.{t}");
		assert_eq!(interpret_as(&parse(&e), &dt), Ok("[]".to_owned()));
		e = format!("\\s.s ({t}) ({e})");
		assert_eq!(interpret_as(&parse(&e), &dt), Ok("[true]".to_owned()));
		e = format!("\\s.s ({f}) ({e})");
		assert_eq!(
			interpret_as(&parse(&e), &dt),
			Ok("[false, true]".to_owned())
		);
		e = format!("\\s.s ({t}) ({e})");
		assert_eq!(
			interpret_as(&parse(&e), &dt),
			Ok("[true, false, true]".to_owned())
		);
		e = format!("\\s.s ({t}) ({e})");
		assert_eq!(
			interpret_as(&parse(&e), &dt),
			Ok("[true, true, false, true]".to_owned())
		);
	}

	#[test]
	fn test_interpret_as_list_church() {
		let dt = DataType::List(Box::new(DataType::ChurchNumeral));
		let mut e = format!("\\_.(\\t.\\f.t)");
		assert_eq!(interpret_as(&parse(&e), &dt), Ok("[]".to_owned()));
		e = format!("\\s.s (\\s.\\z.s(s(z))) ({e})");
		assert_eq!(interpret_as(&parse(&e), &dt), Ok("[2]".to_owned()));
		e = format!("\\s.s (\\s.\\z.s(z)) ({e})");
		assert_eq!(interpret_as(&parse(&e), &dt), Ok("[1, 2]".to_owned()));
		e = format!("\\s.s (\\s.\\z.z) ({e})");
		assert_eq!(interpret_as(&parse(&e), &dt), Ok("[0, 1, 2]".to_owned()));
		e = format!("\\s.s (\\s.\\z.s(z)) ({e})");
		assert_eq!(interpret_as(&parse(&e), &dt), Ok("[1, 0, 1, 2]".to_owned()));
		e = format!("\\s.s (\\s.\\z.s(s(z))) ({e})");
		assert_eq!(
			interpret_as(&parse(&e), &dt),
			Ok("[2, 1, 0, 1, 2]".to_owned())
		);
	}
}
