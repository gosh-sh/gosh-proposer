#!/bin/bash
set -e
set -o pipefail
set -x

gosh-cli config --is_json true -e https://sh.network.gosh.sh

PROP_ADDRESS="0:4594b86ef8ea833d5230d6082524db058bd04373e1888ea8cb33830eb81fc1c9"
PUBKEY=$(cat keys.json | jq  -r .public)
gosh-cli -j callx --addr $PROP_ADDRESS --abi ../contracts/l2/proposal_test.abi.json  -m setvdict --key "0x$PUBKEY"
gosh-cli -j callx --addr $PROP_ADDRESS --abi ../contracts/l2/proposal_test.abi.json --keys keys.json -m setVote --id 0
