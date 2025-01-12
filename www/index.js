import * as lambda from "lambda";

const BASE_URL = getBaseUrl();
const DESCRIPTION_ELEM = document.getElementById("exercise-description");
const INPUT_TEXTAREA = document.getElementById("input-block");
const OUTPUT_TEXTAREA = document.getElementById("output-block");
const EVAL_BTN = document.getElementById("eval-btn");
const CLEAR_BTN = document.getElementById("clear-btn");
const SHOW_ANSWER_BTN = document.getElementById("show-answer-btn");

const url = new URL(window.location.href);

const exercise = await getExercise(url.searchParams.get("ex"));
if (exercise) {
	DESCRIPTION_ELEM.innerHTML = exercise.description;
	if (exercise.answer) {
		SHOW_ANSWER_BTN.style.visibility = "visible";
		SHOW_ANSWER_BTN.addEventListener("click", () => {
			INPUT_TEXTAREA.value = exercise.answer;
		});
	} else {
		SHOW_ANSWER_BTN.style.visibility = "hidden";
	}
} else {
	url.searchParams.delete("ex");
	window.history.replaceState(null, null, url);
	INPUT_TEXTAREA.value = `{:: church }
succ 3
where succ = \\n.\\s.\\z.s(n s z)
where    3 = \\s.\\z.s(s(s(z))) { 3 }`;
}

INPUT_TEXTAREA.addEventListener("keydown", (e) => {
	if (e.ctrlKey && e.key === "Enter") {
		evaluateExpression();
	}
});

EVAL_BTN.addEventListener("click", () => {
	evaluateExpression();
});

CLEAR_BTN.addEventListener("click", () => {
	INPUT_TEXTAREA.value = "";
});

function evaluateExpression() {
	OUTPUT_TEXTAREA.value = "...";
	const e = INPUT_TEXTAREA.value;
	OUTPUT_TEXTAREA.value = lambda.eval_lambda(e);
}

function getBaseUrl() {
	let origin = window.location.origin;
	if (origin.endsWith("/")) {
		origin = origin.slice(0, origin.length - 1);
	}
	let pathname = window.location.pathname;
	if (pathname.startsWith("/")) {
		pathname = pathname.slice(1, pathname.length);
	}
	let url = `${origin}/${pathname}`;
	if (url.endsWith("/")) {
		url = url.slice(0, url.length - 1);
	}
	return url;
}

async function getExercise(exercise) {
	if (!exercise || !/^[a-z\-]+/.test(exercise)) {
		return null;
	}

	try {
		const descriptionResponse = await fetch(`${BASE_URL}/exercises/${exercise}.description.html`);
		if (!descriptionResponse.ok) {
			return null;
		}
		const description = await descriptionResponse.text();

		const answerResponse = await fetch(`${BASE_URL}/exercises/${exercise}.answer.txt`);
		if (!answerResponse.ok) {
			return { description: description, answer: null };
		}
		const answer = await answerResponse.text();

		return { description: description, answer: answer };
	} catch (error) {
		console.error(error);
		return null;
	}
}
