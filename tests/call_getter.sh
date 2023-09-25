#!/bin/bash
set -e
set -o pipefail
set -x

ELOCK_ADDRESS="0x6C720016c9310525f9681acFD7dc6B6034CE59dF"

cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposalList()"
#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposal(uint256)" "0xbcf7a106017c5efcf7be95d5870eb42606c759005f6fe4d693ae6df5fc412832"