#!/bin/bash

# Usage: Run `bash abi-creator.sh` in the root of the metadata folder.

sol_directory="sol"
abi_directory="abi"

for p in $sol_directory/*; do
    file=$(basename $p)
    file_without_extension="${file%.*}.json"
    solc $sol_directory/$file --combined-json abi,devdoc,hashes --overwrite --json-indent 2 > $abi_directory/$file_without_extension
done
