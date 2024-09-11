#!/bin/bash

set -ue

# Move to repo root
cd "$(git rev-parse --show-toplevel)"

# Build
git clean -xdi
wasm-pack build --release
cd www
npm install
npm run start
