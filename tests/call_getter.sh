#!/bin/bash
set -e
set -o pipefail
set -x

ELOCK_ADDRESS="0xe3a6abA818aeB4EE28bcC2944F173133B08c2283"

cast call --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "getProposalList()"
