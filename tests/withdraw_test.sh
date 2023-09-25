#!/bin/bash
set -e
set -o pipefail
set -x

. ./env.deposit

ETH_DESTINATION=0x8F59CcE81C33846d14e5A95C400B9414764e6A98

echo $TOKEN_WALLET_ADDRESS
TOKEN_BALANCE=$(gosh-cli runx --addr $TOKEN_WALLET_ADDRESS --abi ../contracts/l2/TONTokenWallet.abi -m getDetails| jq -r .balance)
gosh-cli callx --addr $TOKEN_WALLET_ADDRESS --abi ../contracts/l2/TONTokenWallet.abi --keys wallet.keys.json -m burn_tokens --_answer_id 0 --to $ETH_DESTINATION --tokens $TOKEN_BALANCE
