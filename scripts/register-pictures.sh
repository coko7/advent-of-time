#!/usr/bin/env bash

json_file="data/picures-generated-$(date +%Y%m%d_%H%M%S).json"
echo '[]' > "$json_file"

pictures=$(fd . -e jpg data/raw/)
id=1

for picture in $pictures; do
    filename=$(basename "$picture")
    computed_path="day-pics/$filename"

    datetime=$(exiftool -DateTimeOriginal -s -s -s "$picture")

    date=${datetime%% *}   # Extract part before first space (date)
    date=$(echo "$date" | tr ':' '/')
    time=${datetime##* }   # Extract part after last space (time)
    time=$(echo "$time" | cut -d ':' -f1,2)

    entry=$(jq --null-input \
        --argjson id "$id" \
        --arg path "$computed_path" \
        --arg date "$date" \
        --arg time "$time" \
        '{
            id: $id, path: $path,
            original_date: $date, time_taken: $time,
            location: ""
        }')

    echo "Created JSON: $entry"
    jq --argjson entry "$entry" '. += [$entry]' "$json_file" > tmp.$$.json \
        && mv tmp.$$.json "$json_file"

    exiftool -all= "$picture"

    ((id++))
done
