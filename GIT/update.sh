#!/bin/sh
cd "$(dirname -- "${0}")"
./rm.sh
./add.sh
cd '../'
git status
