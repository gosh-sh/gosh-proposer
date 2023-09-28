#!/bin/bash
set -e
set -o pipefail
set -x

ELOCK_ADDRESS="0xe975AA4b577EB0E935075F71f4263e8f36e54251"

NEW_VALIDATOR="0x316C51E0e959b633D1f23e38bC99b3C316f812DD"

PREV_VALS=$(cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getValidators()")

cast send --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "proposeChangeValidators(address[])" [$ETH_WALLET_ADDR,$NEW_VALIDATOR] --private-key $ETH_PRIVATE_KEY

cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposedValidators()"

cast send --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "voteForChangeValidators(bool)" true --private-key $ETH_PRIVATE_KEY

cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposedValidators()"

NEW_VALS=$(cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getValidators()")

if [[ "$PREV_VALS" -eq "$NEW_VALS" ]]; then
  echo "Validators did not change"
  exit 1
fi

echo "Validators were changed successfully"
