#!/bin/bash

set -x

for file in $1/*.aeon; do
    python3 inline_constants.py "$file" > "${file%.aeon}.inlined.aeon"
    rm "$file"
done