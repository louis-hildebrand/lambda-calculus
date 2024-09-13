import * as lambda from "lambda";

const BASE_URL = getBaseUrl();

const url = new URL(window.location.href);

const exercise = await getExercise(url.searchParams.get("ex"));
if (exercise) {
	document.getElementById("exercise-description").innerHTML = exercise.description;
	const showAnswerBtn = document.getElementById("show-answer-btn");
	if (exercise.answer) {
		showAnswerBtn.style.visibility = "visible";
		showAnswerBtn.addEventListener("click", () => {
			document.getElementById("input-block").value = exercise.answer;
		});
	} else {
		showAnswerBtn.style.visibility = "hidden";
	}
} else {
	url.searchParams.delete("ex");
	window.history.replaceState(null, null, url);
	document.getElementById("input-block").value = `succ n
where succ = \\n.\\s.\\z.s(n s z)
where    n = \\s.\\z.s(s(s(z))) { 3 }`;
}

document.getElementById("eval-btn").addEventListener("click", () => {
	document.getElementById("output-block").value = "...";
	const e = document.getElementById("input-block").value;
	document.getElementById("output-block").value = lambda.eval_lambda(e);
});

document.getElementById("clear-btn").addEventListener("click", () => {
	document.getElementById("input-block").value = "";
});

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
