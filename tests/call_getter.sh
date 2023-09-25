#!/bin/bash
set -e
set -o pipefail

ELOCK_ADDRESS="0xc86bc889661BB56fb823E9400308C5FA6a76bD0d"

#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getValidators()"

cast send --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "voteForWithdrawal(uint256)" "0xCE3BA8D3F286231A624865407EA86A3319CBD6F9A79C0F56AD4D557CCF3B89DA" --private-key $ETH_PRIVATE_KEY