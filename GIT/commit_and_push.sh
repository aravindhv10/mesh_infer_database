#!/bin/sh
cd "$(dirname -- "${0}")/.."
"${HOME}/SSH/K/P/setup.sh"
git commit -m 'Mostly done writing file chunks'
git push
