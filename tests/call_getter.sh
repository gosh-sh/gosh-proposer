#!/bin/bash
set -e
set -o pipefail
set -x

ELOCK_ADDRESS="0x69736886754698f0B7335B8b7505C6A169D78a5f"
ETH_URL="https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a"
ERC20_ROOT="0xc21d97673B9E0B3AA53a06439F71fDc1facE393B"

#cast send --rpc-url $ETH_URL $ELOCK_ADDRESS "deposit(uint256)" "0xc79c0cec1e233df9c0f8ba150391b0ad628b04214c5bad2fbaef94fd2432264c" --private-key $ETH_PRIVATE_KEY --value 0.0002ether
#cast call --rpc-url $ETH_URL $ELOCK_ADDRESS "lastProcessedL2Block()"
#cast call --rpc-url $ETH_URL $ELOCK_ADDRESS "trxDepositCount()"
#cast call --rpc-url $ETH_URL $ELOCK_ADDRESS "trxWithdrawCount()"
#cast call --rpc-url $ETH_URL $ELOCK_ADDRESS "totalSupply()"
#cast call --rpc-url $ETH_URL $ELOCK_ADDRESS "getProposalList()"
#cast call --rpc-url $ETH_URL $ELOCK_ADDRESS "getProposal(uint256)" "0xfc45f312b36d57a1713db98acf72dde59ec2ce70e749948ee37488c566677ffd"

#cast call --rpc-url $ETH_URL $ELOCK_ADDRESS "getValidators()"

#cast call --rpc-url $ETH_URL $ERC20_ROOT "name()"
#cast call --rpc-url $ETH_URL $ERC20_ROOT "symbol()"
#cast call --rpc-url $ETH_URL $ERC20_ROOT "decimals()"
cast call --rpc-url $ETH_URL "getERC20Approvement"