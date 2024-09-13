use lambda::eval_lambda;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
	assert_eq!(1 + 1, 2);
}

#[wasm_bindgen_test]
fn test_or() {
	let defs = "
		where || = \\a.\\b.a T b
		where  F = \\a.\\b.b
		where  T = \\a.\\b.a"
		.trim();
	let e = |x| format!("{x}\n{defs}");
	let f = "\\a.\\b.b";
	let t = "\\a.\\b.a";

	assert_eq!(eval_lambda(&e("|| F F")), f);
	assert_eq!(eval_lambda(&e("|| F T")), t);
	assert_eq!(eval_lambda(&e("|| T F")), t);
	assert_eq!(eval_lambda(&e("|| T T")), t);

	assert_eq!(eval_lambda(&e("{:: expr } || F F")), f);
	assert_eq!(eval_lambda(&e("{:: expr } || F T")), t);
	assert_eq!(eval_lambda(&e("{:: expr } || T F")), t);
	assert_eq!(eval_lambda(&e("{:: expr } || T T")), t);

	assert_eq!(eval_lambda(&e("{:: bool } || F F")), "false");
	assert_eq!(eval_lambda(&e("{:: bool } || F T")), "true");
	assert_eq!(eval_lambda(&e("{:: bool } || T F")), "true");
	assert_eq!(eval_lambda(&e("{:: bool } || T T")), "true");
}

#[wasm_bindgen_test]
fn test_and() {
	let defs = "
		where && = \\a.\\b.a b F
		where  F = \\a.\\b.b
		where  T = \\a.\\b.a"
		.trim();
	let e = |x| format!("{x}\n{defs}");
	let f = "\\a.\\b.b";
	let t = "\\a.\\b.a";

	assert_eq!(eval_lambda(&e("&& F F")), f);
	assert_eq!(eval_lambda(&e("&& F T")), f);
	assert_eq!(eval_lambda(&e("&& T F")), f);
	assert_eq!(eval_lambda(&e("&& T T")), t);

	assert_eq!(eval_lambda(&e("{:: expr } && F F")), f);
	assert_eq!(eval_lambda(&e("{:: expr } && F T")), f);
	assert_eq!(eval_lambda(&e("{:: expr } && T F")), f);
	assert_eq!(eval_lambda(&e("{:: expr } && T T")), t);

	assert_eq!(eval_lambda(&e("{:: bool } && F F")), "false");
	assert_eq!(eval_lambda(&e("{:: bool } && F T")), "false");
	assert_eq!(eval_lambda(&e("{:: bool } && T F")), "false");
	assert_eq!(eval_lambda(&e("{:: bool } && T T")), "true");
}

fn make_church_num(n: usize) -> String {
	if n == 0 {
		"\\a.\\b.b".to_owned()
	} else {
		let mut body = "a b".to_owned();
		for _ in 1..n {
			body = format!("a ({body})");
		}
		format!("\\a.\\b.{body}")
	}
}

#[wasm_bindgen_test]
fn test_make_church_num() {
	let church_numerals = [
		"\\a.\\b.b",
		"\\a.\\b.a b",
		"\\a.\\b.a (a b)",
		"\\a.\\b.a (a (a b))",
		"\\a.\\b.a (a (a (a b)))",
		"\\a.\\b.a (a (a (a (a b))))",
		"\\a.\\b.a (a (a (a (a (a b)))))",
		"\\a.\\b.a (a (a (a (a (a (a b))))))",
		"\\a.\\b.a (a (a (a (a (a (a (a b)))))))",
		"\\a.\\b.a (a (a (a (a (a (a (a (a b))))))))",
		"\\a.\\b.a (a (a (a (a (a (a (a (a (a b)))))))))",
	];
	for (i, e) in church_numerals.iter().enumerate() {
		assert_eq!(make_church_num(i), e.to_owned());
	}
}

#[wasm_bindgen_test]
fn test_succ() {
	let defs = "where succ = \\n.\\s.\\z.s(n s z)";

	for n in 0..100 {
		let n_expr = make_church_num(n);
		let succ_expr = make_church_num(n + 1);
		assert_eq!(eval_lambda(&format!("succ ({n_expr})\n{defs}")), succ_expr);
		assert_eq!(
			eval_lambda(&format!("{{:: expr}} succ ({n_expr})\n{defs}")),
			succ_expr
		);
		assert_eq!(
			eval_lambda(&format!("{{:: church }} succ ({n_expr})\n{defs}")),
			(n + 1).to_string()
		);
	}
}

#[wasm_bindgen_test]
fn test_plus() {
	let defs = "
		where    + = \\x.\\y.x succ y
		where succ = \\n.\\s.\\z.s(n s z)"
		.trim();

	for n in 0..25 {
		for m in 0..25 {
			let n_expr = make_church_num(n);
			let m_expr = make_church_num(m);
			let sum_expr = make_church_num(n + m);
			assert_eq!(
				eval_lambda(&format!("+ ({n_expr}) ({m_expr})\n{defs}")),
				sum_expr
			);
			assert_eq!(
				eval_lambda(&format!("{{:: expr }} + ({n_expr}) ({m_expr})\n{defs}")),
				sum_expr
			);
			assert_eq!(
				eval_lambda(&format!("{{:: church }} + ({n_expr}) ({m_expr})\n{defs}")),
				(n + m).to_string()
			);
		}
	}
}
