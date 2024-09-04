import * as lambda from "lambda";

document.getElementById("input-block").innerHTML = `succ n
where succ = \\n.\\s.\\z.s(n s z)
where    n = \\s.\\z.s(s(s(z))) { 3 }`;

document.getElementById("eval-btn").addEventListener("click", () => {
	const e = document.getElementById("input-block").value;
	document.getElementById("output-block").innerText = lambda.eval_lambda(e);
});
