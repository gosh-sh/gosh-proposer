#!/bin/bash
set -e
set -o pipefail
set -x

# NOTE: to run this test you need to export your eth wallet to ETH_WALLET_ADDR
# and your eth private key to ETH_PRIVATE_KEY

TEST_TRACE="/home/user/GOSH/gosh-proposer/tests/trace.log"
ETH_URL="https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a"
GOSH_URL="https://sh.network.gosh.sh"

# Generate keypair
gosh-cli genphrase --dump keys.json
PUBKEY="0x$(cat keys.json | jq  -r .public)"
echo "export PUBKEY=$PUBKEY" > $TEST_TRACE

# go to the repo root
cd ..

# Query last blocks
LAST_BLOCKS=$(cargo run -p withdraw_proposal_checker --release  -- get_last_blocks)
echo "LAST_BLOCKS=$LAST_BLOCKS"
LAST_GOSH_BLOCK=$(echo $LAST_BLOCKS | jq -r .gosh.id )
LAST_ETH_BLOCK=$(echo $LAST_BLOCKS | jq -r .eth.hash )
echo "export LAST_GOSH_BLOCK=$LAST_GOSH_BLOCK" >> $TEST_TRACE
echo "export LAST_ETH_BLOCK=$LAST_ETH_BLOCK" >> $TEST_TRACE


# Disable bash trace for not to show private keys
set +x

# go to l1 root
cd contracts/l1/

# deploy Elock
ETH_CONTRACT_ADDRESS=$(forge create --rpc-url $ETH_URL --private-key $ETH_PRIVATE_KEY src/Elock.sol:Elock --constructor-args $LAST_GOSH_BLOCK $ETH_WALLET_ADDR [$ETH_WALLET_ADDR] | grep "Deployed to: " | cut -d ' ' -f 3)
echo "export ETH_CONTRACT_ADDRESS=$ETH_CONTRACT_ADDRESS" >> $TEST_TRACE
# deposit value to Elock
cast send --rpc-url $ETH_URL $ETH_CONTRACT_ADDRESS "deposit(uint256)" $PUBKEY --private-key $ETH_PRIVATE_KEY --value 0.02ether

# top up elock balance
cast send --rpc-url $ETH_URL $ETH_CONTRACT_ADDRESS --private-key $ETH_PRIVATE_KEY --value 0.01ether

# Enable bash trace
set -x

# go back to the tests dir
cd ../../tests

# set up network
gosh-cli config --is_json true -e $GOSH_URL

# deploy Checker
cp ../contracts/l2/checker.tvc ../contracts/l2/checker2.tvc
CHECKER_ADDRESS=$(gosh-cli -j genaddr --save --abi ../contracts/l2/checker.abi.json --setkey keys.json ../contracts/l2/checker2.tvc | jq .raw_address | cut -d '"' -f 2)
echo "export CHECKER_ADDRESS=$CHECKER_ADDRESS" >> $TEST_TRACE
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
echo "export ROOT_ADDRESS=$ROOT_ADDRESS" >> $TEST_TRACE

# set root in checker
gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys keys.json --addr $CHECKER_ADDRESS -m setRootContract --root $ROOT_ADDRESS

CODE_WALLET=$(gosh-cli -j decode stateinit --tvc ../contracts/l2/TONTokenWallet.tvc | jq .code | cut -d '"' -f 2)
gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 1000000000000 --bounce false --allBalance false --payload ""  --dest $ROOT_ADDRESS
gosh-cli -j deployx --abi ../contracts/l2/RootTokenContract.abi --keys root_keys.json ../contracts/l2/RootTokenContract2.tvc --name "geth" --symbol "gth" --decimals 18 --root_pubkey $ROOT_PUBKEY --root_owner null --total_supply 0 --checker $CHECKER_ADDRESS --oldroot_ null --newroot_ null
gosh-cli -j callx --abi ../contracts/l2/RootTokenContract.abi --keys root_keys.json --addr $ROOT_ADDRESS -m setWalletCode --wallet_code $CODE_WALLET --_answer_id 0
rm ../contracts/l2/RootTokenContract2.tvc

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

# Get checker status
gosh-cli -j runx --addr $CHECKER_ADDRESS --abi ../contracts/l2/checker.abi.json -m getStatus

# get token wallet address
TOKEN_WALLET_ADDRESS=$(gosh-cli runx --addr $ROOT_ADDRESS --abi ../contracts/l2/RootTokenContract.abi -m getWalletAddress --owner null --pubkey $PUBKEY | jq -r .value0)
echo "export TOKEN_WALLET_ADDRESS=$TOKEN_WALLET_ADDRESS" >> $TEST_TRACE

# check token wallet balance
TOKEN_BALANCE=$(gosh-cli runx --addr $TOKEN_WALLET_ADDRESS --abi ../contracts/l2/TONTokenWallet.abi -m getDetails| jq -r .balance)
if [[ "$TOKEN_BALANCE" -eq "0" ]]; then
  echo "Wrong balance"
  exit 1
fi


# get root token wallet address
ROOT_TOKEN_WALLET_ADDRESS=$(gosh-cli runx --addr $ROOT_ADDRESS --abi ../contracts/l2/RootTokenContract.abi -m getWalletAddress --owner null --pubkey $ROOT_PUBKEY | jq -r .value0)
echo "export ROOT_TOKEN_WALLET_ADDRESS=$ROOT_TOKEN_WALLET_ADDRESS" >> $TEST_TRACE

# check token wallet balance
ROOT_TOKEN_BALANCE=$(gosh-cli runx --addr $ROOT_TOKEN_WALLET_ADDRESS --abi ../contracts/l2/TONTokenWallet.abi -m getDetails| jq -r .balance)
if [[ "$ROOT_TOKEN_BALANCE" -eq "0" ]]; then
  echo "Wrong root balance"
  exit 1
fi

# Burn tokens
gosh-cli callx --addr $TOKEN_WALLET_ADDRESS --abi ../contracts/l2/TONTokenWallet.abi --keys keys.json -m burnTokens --_answer_id 0 --to $ETH_WALLET_ADDR --tokens $TOKEN_BALANCE

sleep 10

# run withdraw proposal checker
cd ..

# Create proposal in Elock
ROOT_ADDRESS=$ROOT_ADDRESS ETH_CONTRACT_ADDRESS=$ETH_CONTRACT_ADDRESS cargo run -p withdraw_proposal_checker --release  -- create

sleep 10
# Vote for proposal
ROOT_ADDRESS=$ROOT_ADDRESS ETH_CONTRACT_ADDRESS=$ETH_CONTRACT_ADDRESS make run_withdraw
