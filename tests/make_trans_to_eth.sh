ELOCK_ADDRESS=0xe0cAd8f1Deee00329A2437fCe982b90Dc9e03abd
GOSH_USER_PUBKEY=$(cat wallet.keys.json | jq  -r .public)
VALUE=0.0002ether
cast send --rpc-url https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a --private-key $PRIVATE_KEY $ELOCK_ADDRESS "deposit(uint256)" $GOSH_USER_PUBKEY --value $VALUE