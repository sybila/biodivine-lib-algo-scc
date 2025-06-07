#!/bin/bash

# Print running
set -x 

# Set variables for the input folder and output folder
input_folder="bbm-random-inlined"
output_folder="bbm-random-inlined-delay"
python_script="add_delays.py"

# Create the output folder if it doesn't exist
mkdir -p "$output_folder"

# Loop through all .aeon files in the input folder
for file in "$input_folder"/*.aeon; do
    # Get the base filename (without path and extension)
    filename=$(basename "$file" .aeon)

    # Define the output file name
    output_file="$output_folder/${filename}.delay.aeon"

    # Run the Python script with the input file and redirect output to the output file
    python3 "$python_script" "$file" > "$output_file"
done