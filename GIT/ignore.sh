#!/bin/sh
cd "$(dirname -- "${0}")/.."
(cat './GIT/ignore.list' ; sed 's@^@/@g' './GIT/rm.list') > './.gitignore'
