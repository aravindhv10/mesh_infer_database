#!/bin/sh
cd "$(dirname -- "${0}")/.."
"${HOME}/SSH/K/P/setup.sh"
git commit -m 'Added blake3 dependency'
git push
