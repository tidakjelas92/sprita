#!/bin/bash

# This script optimizes all png files in the dir.

EXPORT_DIRNAME="export"
PNG_FILES=(./*.png)

function command_exists() {
  command -v "$1" > /dev/null
  if [[ $? -ne 0 ]]; then
    return 1
  fi

  return 0
}

if ! command_exists sprita; then
  echo >&2 "[ERROR]: sprita was not found. Check the path."
  exit 1
fi

if [[ $PNG_FILES == "./*.png" ]]; then
  echo "[INFO]: There isn't any png file in this directory."
  exit 0
fi

EXPORT_DIR=./$EXPORT_DIRNAME
if [[ ! -d $EXPORT_DIR ]]; then
  mkdir -p $EXPORT_DIR
fi

for FILE in "${PNG_FILES[@]}"; do
  FILENAME=$(basename "$FILE")
  OUTPUT="./$EXPORT_DIRNAME/$FILENAME"
  sprita -i $FILE -o $OUTPUT -f
  if [[ $? -ne 0 ]]; then
    echo >&2 "Error detected while running script. Aborting."
    exit 1
  fi
done
