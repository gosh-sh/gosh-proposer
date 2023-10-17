#!/bin/bash
set -e
set -o pipefail
set -x

ELOCK_ADDRESS="0x135d03AF576633B0C99FB9F0A0c6Aa9cE8D3C67E"

#cast send --rpc-url https://mainnet.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "deposit(uint256)" "0xc79c0cec1e233df9c0f8ba150391b0ad628b04214c5bad2fbaef94fd2432264c" --private-key $ETH_PRIVATE_KEY --value 0.0002ether
cast call --rpc-url https://mainnet.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "lastProcessedL2Block()"
cast call --rpc-url https://mainnet.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "trxDepositCount()"
cast call --rpc-url https://mainnet.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "trxWithdrawCount()"
cast call --rpc-url https://mainnet.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "totalSupply()"
cast call --rpc-url https://mainnet.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposalList()"
#cast call --rpc-url https://mainnet.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposal(uint256)" "0xfc45f312b36d57a1713db98acf72dde59ec2ce70e749948ee37488c566677ffd"

#cast call --rpc-url https://mainnet.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getValidators()"