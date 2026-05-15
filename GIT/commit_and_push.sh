#!/bin/sh
cd "$(dirname -- "${0}")/.."
"${HOME}/SSH/K/P/setup.sh"
git commit -m 'Fully working file chunk hasher'
git push
