#!/usr/bin/env bats

load '../test_helper/bats-support/load'
load '../test_helper/bats-assert/load'

setup() {
    export CHAIN_ID="test-1"
    export RPC="http://localhost:26657"
    export CONTAINER_NAME="e2e-manifest-ledger-1"
    export WASM_FILE="artifacts/converter.wasm"
    export USER_ADDR="manifest1hj5fveer5cjtn4wd6wstzugjfdxzl0xp8ws9ct"
    export USER2_ADDR="manifest1efd63aw40lxf3n4mhf7dzhjkr453axurm6rp3z"
    export POA_ADMIN_ADDR=$USER_ADDR
    export RATE="2"
    export SOURCE_DENOM="umfx"
    export TARGET_DENOM="factory/manifest1afk9zr2hn2jsac63h4hm60vl9z3e5u69gndzf7c99cqge3vzwjzsfmy9qj/upwr"
    export GAS_PRICES="0.0011umfx"

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

@test "convert" {
    #####
    ##### STORE CONTRACT
    #####
    echo "# --> Storing contract on-chain" >&3
    run docker exec "$CONTAINER_NAME" manifestd tx wasm store \
        "/workspace/$WASM_FILE" \
        --from "$USER_ADDR" \
        --home "/persistent/.manifest" \
        --keyring-backend "test" \
        --gas auto \
        --gas-adjustment 1.4 \
        --gas-prices "$GAS_PRICES" \
        --chain-id "$CHAIN_ID" \
        --node "$RPC" \
        --yes \
        --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +3)
    TX_HASH=$(echo "$JSON_OUTPUT" | jq -r '.txhash')
    assert [ -n "$TX_HASH" ]
    ERR_CODE=$(echo "$JSON_OUTPUT" | jq -r '.code')
    assert [ "$ERR_CODE" -eq 0 ]

    echo "# --> Transaction submitted with hash: $TX_HASH" >&3

    sleep 3

    # Query transaction to get code ID
    run docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" \
        --node "$RPC" \
        --output json

    assert_success

    CODE_ID=$(echo "$output" | tail -n +2 | jq -r '.events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
    assert [ "$CODE_ID" -gt 0 ]

    echo "# --> Contract uploaded successfully with code ID: $CODE_ID" >&3

    # Verify code exists
    run docker exec "$CONTAINER_NAME" manifestd query wasm code-info "$CODE_ID" \
        --node "$RPC" \
        --output json

    assert_success

    CREATOR=$(echo "$output" | tail -n +2 | jq -r '.code_info.creator')
    assert [ -n "$CREATOR" ]

    #####
    ##### INSTANTIATE CONTRACT
    #####
    INIT_MSG='{"admin":"'$USER_ADDR'","poa_admin":"'$POA_ADMIN_ADDR'","rate":"'$RATE'","source_denom":"'$SOURCE_DENOM'","target_denom":"'$TARGET_DENOM'","paused":false}'

    echo "# --> Instantiating contract" >&3
    run docker exec "$CONTAINER_NAME" manifestd tx wasm instantiate "$CODE_ID" \
        "$INIT_MSG" \
        --from "$USER_ADDR" \
        --home "/persistent/.manifest" \
        --keyring-backend "test" \
        --label "converter-test" \
        --gas auto \
        --gas-adjustment 1.4 \
        --gas-prices "$GAS_PRICES" \
        --chain-id "$CHAIN_ID" \
        --node "$RPC" \
        --yes \
        --admin "$USER_ADDR" \
        --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +3)
    TX_HASH=$(echo "$JSON_OUTPUT" | jq -r '.txhash')
    assert [ -n "$TX_HASH" ]
    ERR_CODE=$(echo "$JSON_OUTPUT" | jq -r '.code')
    assert [ "$ERR_CODE" -eq 0 ]

    echo "# --> Transaction submitted with hash: $TX_HASH" >&3

    sleep 3

    # Query transaction to get code ID
    run docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" \
      --node "$RPC" \
      --output json

    assert_success

    CONTRACT_ADDR=$(echo "$output" | tail -n +2 | jq -r '.events[] | select(.type=="wasm") | .attributes[] | select(.key=="_contract_address") | .value')
    assert [ -n "$CONTRACT_ADDR" ]

    #####
    ##### GRANT AUTHZ PWR MINTING CAPABILITIES TO CONTRACT
    #####
    echo "# --> Setting PWR minting permissions to contract" >&3
    run docker exec "$CONTAINER_NAME" manifestd tx authz grant $CONTRACT_ADDR generic \
      --msg-type "/osmosis.tokenfactory.v1beta1.MsgMint"  \
      --from "$USER_ADDR" \
      --home "/persistent/.manifest" \
      --keyring-backend "test" \
      --gas auto \
      --gas-adjustment 1.8 \
      --gas-prices "$GAS_PRICES" \
      --chain-id "$CHAIN_ID" \
      --node "$RPC" \
      --yes \
      --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +3)
    TX_HASH=$(echo "$JSON_OUTPUT" | jq -r '.txhash')
    assert [ -n "$TX_HASH" ]
    ERR_CODE=$(echo "$JSON_OUTPUT" | jq -r '.code')
    assert [ "$ERR_CODE" -eq 0 ]

    echo "# --> Transaction submitted with hash: $TX_HASH" >&3

    sleep 3

    # Query transaction to get code ID
    run docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" \
      --node "$RPC" \
      --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +2)
    ERR_CODE=$(echo "$JSON_OUTPUT" | jq -r '.code')
    assert [ "$ERR_CODE" -eq 0 ]

    #####
    ##### GRANT AUTHZ MFX BURNING CAPABILITIES TO CONTRACT
    #####
    echo "# --> Setting MFX burning permissions to contract" >&3
    run docker exec "$CONTAINER_NAME" manifestd tx authz grant $CONTRACT_ADDR generic \
      --msg-type "/liftedinit.manifest.v1.MsgBurnHeldBalance"  \
      --from "$USER_ADDR" \
      --home "/persistent/.manifest" \
      --keyring-backend "test" \
      --gas auto \
      --gas-adjustment 1.8 \
      --gas-prices "$GAS_PRICES" \
      --chain-id "$CHAIN_ID" \
      --node "$RPC" \
      --yes \
      --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +3)
    TX_HASH=$(echo "$JSON_OUTPUT" | jq -r '.txhash')
    assert [ -n "$TX_HASH" ]
    ERR_CODE=$(echo "$JSON_OUTPUT" | jq -r '.code')
    assert [ "$ERR_CODE" -eq 0 ]

    echo "# --> Transaction submitted with hash: $TX_HASH" >&3

    sleep 3

    # Query transaction to get code ID
    run docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" \
      --node "$RPC" \
      --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +2)
    ERR_CODE=$(echo "$JSON_OUTPUT" | jq -r '.code')
    assert [ "$ERR_CODE" -eq 0 ]

    #####
    ##### CONVERT MXF TO PWR
    #####
    CONVERT_MSG='{"convert":{}}'
    echo "# --> Converting 1 MFX to $RATE PWR" >&3
    run docker exec "$CONTAINER_NAME" manifestd tx wasm execute "$CONTRACT_ADDR" \
        "$CONVERT_MSG" \
        --from "$USER2_ADDR" \
        --home "/persistent/.manifest" \
        --keyring-backend "test" \
        --amount "1000000$SOURCE_DENOM" \
        --gas auto \
        --gas-adjustment 1.4 \
        --gas-prices "$GAS_PRICES" \
        --chain-id "$CHAIN_ID" \
        --node "$RPC" \
        --yes \
        --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +3)
    TX_HASH=$(echo "$JSON_OUTPUT" | jq -r '.txhash')
    assert [ -n "$TX_HASH" ]
    ERR_CODE=$(echo "$JSON_OUTPUT" | jq -r '.code')
    assert [ "$ERR_CODE" -eq 0 ]

    echo "# --> Transaction submitted with hash: $TX_HASH" >&3

    sleep 3

    # Query transaction to get code ID
    run docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" \
      --node "$RPC" \
      --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +2)
    ERR_CODE=$(echo "$JSON_OUTPUT" | jq -r '.code')
    assert [ "$ERR_CODE" -eq 0 ]

    run docker exec "$CONTAINER_NAME" manifestd query bank balances "$USER2_ADDR" \
      --node "$RPC" \
      --output json

    assert_success

    JSON_OUTPUT=$(echo "$output" | tail -n +2)
    PWR_BALANCE=$(echo "$JSON_OUTPUT" | jq -r '.balances[] | select(.denom=="'"$TARGET_DENOM"'") | .amount')
    assert [ "$PWR_BALANCE" -eq 2000000 ]

    echo "# --> Conversion successful, received $RATE PWR" >&3

    # TODO: Change config, Update Admin
}
