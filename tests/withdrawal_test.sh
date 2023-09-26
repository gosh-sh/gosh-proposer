#!/bin/bash
set -e
set -o pipefail

cd ../contracts/l1/
ELOCK_ADDRESS=$(forge create --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a --private-key $ETH_PRIVATE_KEY src/Elock.sol:Elock --constructor-args "5cbc704d975acc44f8af56c9bcc6f90e3900ac59bb572a9b07a52f2ac5289124" ["0xA2Cd57002cD089b7166ad40Bb1402664afc64067"] | grep "Deployed to: " | cut -d ' ' -f 3)

echo "ELOCK_ADDRESS=$ELOCK_ADDRESS"

cast send --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a $ELOCK_ADDRESS "deposit(uint256)" "0xc79c0cec1e233df9c0f8ba150391b0ad628b04214c5bad2fbaef94fd2432264c" --private-key $ETH_PRIVATE_KEY --value 0.0002ether

#ELOCK_ADDRESS="0x89a96E0aD2c647f775BF9dADCB01a87c3C213983"

cd -
cd ..

ETH_CONTRACT_ADDRESS=$ELOCK_ADDRESS make run_withdraw

