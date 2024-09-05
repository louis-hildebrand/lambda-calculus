#!/bin/bash

set -ue

# Move to repo root
cd "$(git rev-parse --show-toplevel)"

head_sha="$(git rev-parse HEAD)"
main_sha="$(git rev-parse main)"
if [[ "${head_sha}" != "${main_sha}" ]]; then
	echo "The current commit is not the one pointed to by the main branch. Please check out the main branch before deploying."
	exit 1
fi

untracked_files="$(git ls-files --exclude-standard --others)"
if [[ "${untracked_files}" != "" ]]; then
	echo "${untracked_files}"
	echo "There are untracked files. Please delete, stash, or commit them before continuing."
	exit 1
fi

# Build
git clean -xdi
wasm-pack build --release
cd www
npm install
npm run build

# Manually copy CSS files into www/dist/ because it's not immediately obvious
# how to do that directly with webpack.
cp ./*.css ./dist

# Deploy to gh-pages branch, overwriting previous deployment
if git rev-parse --verify gh-pages; then
	git branch --delete --force gh-pages
fi
git switch --orphan gh-pages
find . -maxdepth 1 ! -name . ! -name dist              | xargs rm -r
cd ..
find . -maxdepth 1 ! -name . ! -name  www ! -name .git | xargs rm -r
mv www/dist/* .
rmdir www/dist
rmdir www
# This makes the build process a bit faster on GitHub
touch .nojekyll
git add .
git commit -m "Deploy"
git push --set-upstream --force origin HEAD
git switch main
