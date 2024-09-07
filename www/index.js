import * as lambda from "lambda";

document.getElementById("input-block").innerHTML = `succ n
where succ = \\n.\\s.\\z.s(n s z)
where    n = \\s.\\z.s(s(s(z))) { 3 }`;

document.getElementById("eval-btn").addEventListener("click", () => {
	document.getElementById("output-block").innerText = "...";
	const e = document.getElementById("input-block").value;
	const datatype = document.getElementById("interpret-as").value;
	document.getElementById("output-block").innerText = lambda.eval_lambda(e, datatype);
});
