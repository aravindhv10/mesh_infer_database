#!/bin/sh
cd "$(dirname -- "${0}")/.."
"${HOME}/SSH/K/P/setup.sh"
git commit -m 'More work on getting pieces of file'
git push
