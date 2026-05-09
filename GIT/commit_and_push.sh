#!/bin/sh
cd "$(dirname -- "${0}")/.."
"${HOME}/SSH/K/P/setup.sh"
git commit -m 'More refinement of git scripts'
git push
