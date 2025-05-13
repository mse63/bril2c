files=$(find ./benchmarks -name "*.bril" -type f -exec wc -l {} + | sort -n | grep -v ' total$' | awk '{print $2}')
# Find all .bril files in the current directory
export RUSTFLAGS="-Awarnings"
for file in $files; do

    filename=$(basename -- "$file" .bril)

    args_line=$(grep -m 1 -E '^(#ARGS:|# ARGS:)' "$file")

    if [[ "$args_line" =~ ^#\ ?ARGS:\ (.*) ]]; then
        args=${BASH_REMATCH[1]}
    else
        args=""
    fi

    echo "Trying: $file"

    # Run both programs with or without arguments
    output1=$(cat "$file" | bril2json | brili $args | sed 's/[[:space:]]\+$//')
    output2=$(cat "$file" | bril2json | cargo run --quiet | gcc -x c -o temp - && ./temp $args | sed 's/[[:space:]]\+$//')


    if [ "$output1" == "$output2" ]; then
        echo "Match: $file"
    else
        echo "Mismatch: $file"
        echo "p1 output:"
        echo "$output1"
        echo "p2 output:"
        echo "$output2"
    fi
done
