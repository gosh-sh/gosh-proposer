#!/bin/bash
set -e
set -o pipefail
set -x

gosh-cli config --is_json true -e https://sh.network.gosh.sh

ADDRESS=$(gosh-cli -j genaddr --save --abi ../contracts/l2/checker.abi.json --genkey keys.json ../contracts/l2/checker.tvc | jq .raw_address | cut -d '"' -f 2)
echo "address=$ADDRESS"
gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 100000000000 --bounce false --allBalance false --payload ""  --dest $ADDRESS
gosh-cli -j deployx --abi ../contracts/l2/checker.abi.json --keys keys.json ../contracts/l2/checker.tvc
PROP_CODE=$(gosh-cli -j decode stateinit --tvc ../contracts/l2/proposal.tvc | jq .code | cut -d '"' -f 2)
gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys keys.json --addr $ADDRESS -m setProposalCode --code $PROP_CODE