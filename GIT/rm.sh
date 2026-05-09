#!/bin/sh
cd "$(dirname -- "${0}")/.."
sed 's@^@"./@g ; s@$@"@g' './GIT/rm.list' | tr '\n' ' ' | sed 's@^@"rm" "-vf" "--" @g' | sh
