#!/bin/bash

# Directories
wasm_dir="tests/wasm"
expect_dir="tests/expect"

# Colors
GREEN="\033[0;32m"
RED="\033[0;31m"
NC="\033[0m" # No color

# Arrays to store passed and failed tests
passed_tests=()
failed_tests=()

# Function to run a wasm file with wasmtime and get stdout and stderr
function run_wasm {
    local wasm_file="$1"
    shift
    local run_args=$@
    if [ -z "$run_args" ]; then
      local output=$(timeout 5 ./wasm-vm --jit "$wasm_file" 2>&1)
    else
      local output=$(timeout 5 ./wasm-vm --jit -a $run_args "$wasm_file" 2>&1)
    fi
    if [ $? -ne 0 ]; then
      echo "timeout"
    else
      echo "$output"
    fi
}

function check_output {
    local wasm_file="$1"
    shift
    local expected_output="$1"
    shift
    local run_args="$@"

    local actual_output=$(run_wasm "$wasm_file" $run_args)

    local wasm_filename=$(basename "$wasm_file" .wasm)
    local run_args_print=${run_args:-(void)}

    if [ "$actual_output" == "$expected_output" ]; then
        echo "$wasm_filename (run_args: ${run_args_print}) ... success!"
        passed_tests+=("$wasm_filename (args: ${run_args_print})")
        return 0
    else
        echo "$wasm_filename (run_args: ${run_args_print}) ... fail!"
        echo "  Expected output:"
        echo "  -------------------"
        echo "$expected_output"
        echo "  -------------------"
        echo "  Actual output:"
        echo "  -------------------"
        echo "$actual_output"
        echo "  -------------------"
        failed_tests+=("$wasm_filename (args: ${run_args_print})")
        return 1
    fi
}

num_tests=0
pass=0

# Iterate over all .wasm files in the wasm directory
for wasm_file in "$wasm_dir"/*.wasm; do
    wasm_filename=$(basename "$wasm_file" .wasm)
    
    # Check for corresponding .expect file
    expect_file="$expect_dir/$wasm_filename.expect"
    # Check for corresponding .runs file
    runs_file="$expect_dir/$wasm_filename.runs"
    
    echo "#### Test $((num_tests+1)) ####"
    if [ -f "$expect_file" ]; then
        expected_output=$(cat "$expect_file")
        check_output $wasm_file $expected_output
        if [ $? -eq 0 ]; then
          ((pass++))
        fi
        ((num_tests++))
    elif [ -f "$runs_file" ]; then
        runs_fail=0
        while IFS= read -r line; do
            if [[ "$line" =~ (.*)=(.*) ]]; then
                run_args=${BASH_REMATCH[1]}
                expected_output="${BASH_REMATCH[2]}"
                check_output $wasm_file $expected_output $run_args
                if [ $? -ne 0 ]; then
                  ((runs_fail++))
                fi
            fi
        done < "$runs_file"
        if [ $runs_fail -eq 0 ]; then
          ((pass++))
        fi
        ((num_tests++))
    else
        echo "$wasm_filename... no .expect or .runs file found!"
    fi
    echo ""
done

fail=$((num_tests-pass))

echo ""
echo "============== SUMMARY =============="

# Print passed tests in green
echo -e "${GREEN}Passed Tests:${NC}"
for test in "${passed_tests[@]}"; do
    echo -e "${GREEN}✔ $test${NC}"
done

# Print failed tests in red
echo -e "${RED}Failed Tests:${NC}"
for test in "${failed_tests[@]}"; do
    echo -e "${RED}✘ $test${NC}"
done

echo ""
printf '# Tests: %d | Pass: %d; Fail: %d\n' $num_tests $pass $fail
echo "====================================="
exit $fail
