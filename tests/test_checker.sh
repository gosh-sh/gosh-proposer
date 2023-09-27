#!/bin/bash
set -e
set -o pipefail
set -x

GOSH_URL="https://sh.network.gosh.sh"
ETH_CONTRACT_ADDRESS=0xe2aC76043137F28e913cd66eD895Ab502f991b8B

# Generate keypair
gosh-cli genphrase --dump keys.json
PUBKEY="0x$(cat keys.json | jq  -r .public)"

# Query last blocks
LAST_BLOCKS=$(cargo run -p withdraw_proposal_checker --release  -- get_last_blocks)
echo "LAST_BLOCKS=$LAST_BLOCKS"
LAST_ETH_BLOCK=$(echo $LAST_BLOCKS | jq -r .eth.hash )
echo "export LAST_ETH_BLOCK=$LAST_ETH_BLOCK"

# set up network
gosh-cli config --is_json true -e $GOSH_URL

# deploy Checker
cp ../contracts/l2/checker.tvc ../contracts/l2/checker2.tvc
CHECKER_ADDRESS=$(gosh-cli -j genaddr --save --abi ../contracts/l2/checker.abi.json --setkey keys.json ../contracts/l2/checker2.tvc | jq .raw_address | cut -d '"' -f 2)
echo "export CHECKER_ADDRESS=$CHECKER_ADDRESS"
# ask giver
gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 100000000000 --bounce false --allBalance false --payload ""  --dest $CHECKER_ADDRESS
gosh-cli -j deployx --abi ../contracts/l2/checker.abi.json --keys keys.json ../contracts/l2/checker2.tvc --prevhash $LAST_ETH_BLOCK
rm ../contracts/l2/checker2.tvc

# set proposal code
PROP_CODE=$(gosh-cli -j decode stateinit --tvc ../contracts/l2/proposal_test.tvc | jq .code | cut -d '"' -f 2)
gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys keys.json --addr $CHECKER_ADDRESS -m setProposalCode --code $PROP_CODE


# deploy Root
gosh-cli genphrase --dump root_keys.json
ROOT_PUBKEY="0x$(cat root_keys.json | jq  -r .public)"
cp ../contracts/l2/RootTokenContract.tvc ../contracts/l2/RootTokenContract2.tvc
ROOT_ADDRESS=$(gosh-cli -j genaddr --save --abi ../contracts/l2/RootTokenContract.abi --setkey root_keys.json ../contracts/l2/RootTokenContract2.tvc | jq .raw_address | cut -d '"' -f 2)
echo "export ROOT_ADDRESS=$ROOT_ADDRESS"

# set root in checker
gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys keys.json --addr $CHECKER_ADDRESS -m setRootContract --root $ROOT_ADDRESS

CODE_WALLET=$(gosh-cli -j decode stateinit --tvc ../contracts/l2/TONTokenWallet.tvc | jq .code | cut -d '"' -f 2)
gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 1000000000000 --bounce false --allBalance false --payload ""  --dest $ROOT_ADDRESS
gosh-cli -j deployx --abi ../contracts/l2/RootTokenContract.abi --keys root_keys.json ../contracts/l2/RootTokenContract2.tvc --name "geth" --symbol "gth" --decimals 18 --root_pubkey $ROOT_PUBKEY --root_owner null --total_supply 0 --checker $CHECKER_ADDRESS --oldroot_ null --newroot_ null
gosh-cli -j callx --abi ../contracts/l2/RootTokenContract.abi --keys root_keys.json --addr $ROOT_ADDRESS -m setWalletCode --wallet_code $CODE_WALLET --_answer_id 0
rm ../contracts/l2/RootTokenContract2.tvc

for i in {1..10}
do
  # Get checker status
  PREV_HASH=$(gosh-cli runx --addr $CHECKER_ADDRESS --abi ../contracts/l2/checker.abi.json -m getStatus | jq -r .prevhash)
  # run gosh-proposer to find deposit in ETH and deploy token wallet
  cd ..
  ETH_CONTRACT_ADDRESS=$ETH_CONTRACT_ADDRESS CHECKER_ADDRESS=$CHECKER_ADDRESS make run_proposer
  cd tests

  # get proposal address
  PROP_ADDRESS=$(gosh-cli runx --addr $CHECKER_ADDRESS --abi ../contracts/l2/checker.abi.json -m getAllProposalAddr | jq -r '.value0[0]')
  gosh-cli -j callx --addr $PROP_ADDRESS --abi ../contracts/l2/proposal_test.abi.json  -m setvdict --key $PUBKEY


  # run deposit_proposal_checker to check proposal and vote for it
  cd ..
  VALIDATORS_KEY_PATH=tests/keys.json ETH_CONTRACT_ADDRESS=$ETH_CONTRACT_ADDRESS CHECKER_ADDRESS=$CHECKER_ADDRESS make run_deposit
  cd tests

  sleep 10

  NEW_HASH=$(gosh-cli runx --addr $CHECKER_ADDRESS --abi ../contracts/l2/checker.abi.json -m getStatus | jq -r .prevhash)
  echo $NEW_HASH
  if [[ $NEW_HASH -eq $PREV_HASH ]]; then
    echo "Hash did not change"
    exit 1
  else
    echo "Hash successfully changed"
  fi

  sleep 60
done
