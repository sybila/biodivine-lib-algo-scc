#!/bin/bash

set -x

for file in $1/*.aeon; do
    python3 print_sd.py "$file" > "${file%.aeon}.sd.json"
done