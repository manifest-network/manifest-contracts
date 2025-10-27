#!/usr/bin/env bats

load '../test_helper/bats-support/load'
load '../test_helper/bats-assert/load'

setup() {
    export CHAIN_ID="test-1"
    export RPC="http://localhost:26657"
    export VALIDATOR_KEY="user1"
    export CONTAINER_NAME="e2e-manifest-ledger-1"
    export WASM_FILE="artifacts/converter.wasm"

    # Build optimized contract
    run make build-opt
    assert_success
    assert [ -f "$WASM_FILE" ]

    # Start chain
    run make e2e-chain-up
    assert_success

    # Wait for chain to be ready
    sleep 5
}

teardown() {
    if [ "$BATS_TEST_COMPLETED" = "1" ]; then
        make e2e-chain-down
    fi
}

@test "upload converter contract to chain" {
    # Upload contract
    run docker exec "$CONTAINER_NAME" manifestd tx wasm store \
        "/workspace/$WASM_FILE" \
        --from "$VALIDATOR_KEY" \
        --home "/persistent/.manifest" \
        --keyring-backend "test" \
        --gas auto \
        --gas-adjustment 1.4 \
        --gas-prices "0.0011umfx" \
        --chain-id "$CHAIN_ID" \
        --node "$RPC" \
        --yes \
        --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +3)

    # Extract transaction hash
    TX_HASH=$(echo "$JSON_OUTPUT" | jq -r '.txhash')
    assert [ -n "$TX_HASH" ]

    # Check for error code
    ERR_CODE=$(echo "$JSON_OUTPUT" | jq -r '.code')
    assert [ "$ERR_CODE" -eq 0 ]

    echo "# Transaction submitted with hash: $TX_HASH" >&3

    sleep 3

    # Query transaction to get code ID
    run docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" \
        --node "$RPC" \
        --output json

    assert_success

    CODE_ID=$(echo "$output" | tail -n +2 | jq -r '.events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
    assert [ "$CODE_ID" -gt 0 ]

    echo "# Contract uploaded successfully with code ID: $CODE_ID" >&3

    # Verify code exists
    run docker exec "$CONTAINER_NAME" manifestd query wasm code-info "$CODE_ID" \
        --node "$RPC" \
        --output json

    assert_success

    CREATOR=$(echo "$output" | tail -n +2 | jq -r '.code_info.creator')
    assert [ -n "$CREATOR" ]
}
