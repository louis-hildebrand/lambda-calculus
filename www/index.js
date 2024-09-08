import * as lambda from "lambda";

document.getElementById("input-block").innerHTML = `succ n
where succ = \\n.\\s.\\z.s(n s z)
where    n = \\s.\\z.s(s(s(z))) { 3 }`;

const url = new URL(window.location.href);
const dt = url.searchParams.get("dt");
const VALID_DATA_TYPES = ["expr", "bool", "church"];

if (VALID_DATA_TYPES.includes(dt)) {
	document.getElementById("interpret-as").value = dt;
} else if (!dt) {
	url.searchParams.set("dt", "expr");
	window.history.replaceState(null, null, url);
}

document.getElementById("interpret-as").addEventListener("change", () => {
	url.searchParams.set("dt", document.getElementById("interpret-as").value);
	window.history.replaceState(null, null, url);
});

document.getElementById("eval-btn").addEventListener("click", () => {
	document.getElementById("output-block").innerText = "...";
	const e = document.getElementById("input-block").value;
	const datatype = document.getElementById("interpret-as").value;
	document.getElementById("output-block").innerText = lambda.eval_lambda(e, datatype);
});
