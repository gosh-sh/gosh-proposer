#!/bin/bash
set -e
set -o pipefail
set -x

ETH_PREV_HASH=0x8d945cccc417457135cd5c930af42731de9e9a6013d7cfad032120ea99f9e048
OUT_ENV=env.deposit

gosh-cli config --is_json true -e https://sh.network.gosh.sh

cp ../contracts/l2/checker.tvc ../contracts/l2/checker2.tvc
ADDRESS=$(gosh-cli -j genaddr --save --abi ../contracts/l2/checker.abi.json --genkey keys.json ../contracts/l2/checker2.tvc | jq .raw_address | cut -d '"' -f 2)
echo "export CHECKER_ADDRESS=$ADDRESS" > $OUT_ENV
gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 100000000000 --bounce false --allBalance false --payload ""  --dest $ADDRESS
gosh-cli -j deployx --abi ../contracts/l2/checker.abi.json --keys keys.json ../contracts/l2/checker2.tvc --prevhash $ETH_PREV_HASH
rm ../contracts/l2/checker2.tvc

PROP_CODE=$(gosh-cli -j decode stateinit --tvc ../contracts/l2/proposal_test.tvc | jq .code | cut -d '"' -f 2)
gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys keys.json --addr $ADDRESS -m setProposalCode --code $PROP_CODE

cp ../contracts/l2/RootTokenContract.tvc ../contracts/l2/RootTokenContract2.tvc
ADDRESS_ROOT=$(gosh-cli -j genaddr --save --abi ../contracts/l2/RootTokenContract.abi --setkey keys.json ../contracts/l2/RootTokenContract2.tvc | jq .raw_address | cut -d '"' -f 2)
echo "export ROOT_ADDRESS=$ADDRESS_ROOT" >> $OUT_ENV
gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys keys.json --addr $ADDRESS -m setRootContract --root $ADDRESS_ROOT

CODE_WALLET=$(gosh-cli -j decode stateinit --tvc ../contracts/l2/TONTokenWallet.tvc | jq .code | cut -d '"' -f 2)
PUBKEY=$(cat keys.json | jq  -r .public)
gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 1000000000000 --bounce false --allBalance false --payload ""  --dest $ADDRESS_ROOT
gosh-cli -j deployx --abi ../contracts/l2/RootTokenContract.abi --keys keys.json ../contracts/l2/RootTokenContract2.tvc --name "geth" --symbol "gth" --decimals 18 --root_pubkey "0x$PUBKEY" --root_owner null --total_supply 0 --checker $ADDRESS
gosh-cli -j callx --abi ../contracts/l2/RootTokenContract.abi --keys keys.json --addr $ADDRESS_ROOT -m setWalletCode --wallet_code $CODE_WALLET --_answer_id 0
rm ../contracts/l2/RootTokenContract2.tvc

cd ..
CHECKER_ADDRESS=$ADDRESS make run_proposer

cd tests
PROP_ADDRESS=$(gosh-cli runx --addr $ADDRESS --abi ../contracts/l2/checker.abi.json -m getAllProposalAddr | jq -r '.value0[0]')
gosh-cli -j callx --addr $PROP_ADDRESS --abi ../contracts/l2/proposal_test.abi.json  -m setvdict --key "0x$PUBKEY"


cd ..
CHECKER_ADDRESS=$ADDRESS make run_checker
cd tests


gosh-cli -j runx --addr $ADDRESS --abi ../contracts/l2/checker.abi.json -m getStatus

GOSH_USER_PUBKEY=$(cat wallet.keys.json | jq  -r .public)
TOKEN_WALLET_ADDRESS=$(gosh-cli runx --addr $ADDRESS_ROOT --abi ../contracts/l2/RootTokenContract.abi -m getWalletAddress --owner null --pubkey "0x$GOSH_USER_PUBKEY" | jq -r .value0)
echo "export TOKEN_WALLET_ADDRESS=$TOKEN_WALLET_ADDRESS" >> $OUT_ENV
TOKEN_BALANCE=$(gosh-cli runx --addr $TOKEN_WALLET_ADDRESS --abi ../contracts/l2/TONTokenWallet.abi -m getDetails| jq -r .balance)
if [[ "$TOKEN_BALANCE" != "200000000000000" ]]; then
  echo "Wrong balance"
  exit 1
fi
echo "Success"