#!/bin/bash
set -e
set -o pipefail
set -x

ELOCK_ADDRESS="0x6C720016c9310525f9681acFD7dc6B6034CE59dF"

#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "deposit()"
#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposalList()"
#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposal(uint256)" "0xce3ba8d3f286231a624865407ea86a3319cbd6f9a79c0f56ad4d557ccf3b89da"
#cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getValidators()"