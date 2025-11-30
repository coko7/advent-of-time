#!/usr/bin/env bash

# Find all JPEG images in current dir and resize them to be 1600px
find . -type f \( -iname "*.jpg" -o -iname "*.jpeg" \) -exec mogrify -resize 1600x\> {} +
