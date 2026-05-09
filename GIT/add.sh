#!/bin/sh
cd "$(dirname -- "${0}")/.."
git add '--pathspec-from-file=./GIT/add.list'
