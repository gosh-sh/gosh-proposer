#!/bin/bash
set -e
set -o pipefail
set -x

ELOCK_ADDRESS="0x5CECaf8013491ac6730dFf0726B317922075B5e9"

#cast send --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "deposit(uint256)" "0xc79c0cec1e233df9c0f8ba150391b0ad628b04214c5bad2fbaef94fd2432264c" --private-key $ETH_PRIVATE_KEY --value 0.0002ether
cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "totalSupply()"
#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposal(uint256)" "0xc38ded26357e8ee27f156a007be086dee5a55cc8e6066b62e3eb28a8e8a6b38"

#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getValidators()"