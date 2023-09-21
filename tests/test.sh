#!/bin/bash
set -e
set -o pipefail

gosh-cli config --is_json true -e https://sh.network.gosh.sh

ADDRESS=$(gosh-cli -j genaddr --save --abi solidity/checker.abi.json --setkey keys.json solidity/checker.tvc | jq .raw_address | cut -d '"' -f 2)
echo "address=$ADDRESS"
gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 10000000000 --bounce false --allBalance false --payload ""  --dest $ADDRESS
gosh-cli -j deployx --abi solidity/checker.abi.json --keys keys.json solidity/checker.tvc
