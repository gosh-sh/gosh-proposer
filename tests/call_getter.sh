#!/bin/bash
set -e
set -o pipefail
set -x

ELOCK_ADDRESS="0x3A38504ba6b701af75fEceF3cB3a44E3A3D789dA"

cast send --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "deposit(uint256)" "0xc79c0cec1e233df9c0f8ba150391b0ad628b04214c5bad2fbaef94fd2432264c" --private-key $ETH_PRIVATE_KEY --value 0.0002ether
#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposalList()"
#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposal(uint256)" "0x95169040d163100d6323ec7dccaef564f91c6491bad7e52bebb1425800b64ac2"
#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getValidators()"