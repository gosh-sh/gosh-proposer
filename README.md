Project working scheme

1) Deploy validator ETH wallet and get private key and wallet address
2) Get latest ETH and GOSH blocks:

```bash
withdraw_proposal_checker get_last_blocks
```

3) Deploy ELock contract to the ETH network with latest GOSH block and validator wallet addresses

```bash
forge create --rpc-url $ETH_URL --private-key $ETH_PRIVATE_KEY src/Elock.sol:Elock --constructor-args $LAST_GOSH_BLOCK [$ETH_WALLET_ADDRS] 
```

4) Send some funds to the ELock contract for it to have operational balance

```bash
cast send --rpc-url $ETH_URL $ETH_CONTRACT_ADDRESS --private-key $ETH_PRIVATE_KEY --value <value>
```

5) Deploy checker contract to the GOSH network with latest ETH block
6) Deploy root token contract to the GOSH network and set root in checker
7) Create `.env` file on each validator

Example with comments (better to remove them before usage)
```yaml
# THIS API_KEY should be changed for production   
ETH_NETWORK=wss://sepolia.infura.io/ws/v3/df557e910fb2496e8d854046cbedb99a

GOSH_ENDPOINTS="https://sh.network.gosh.sh"

ETH_FUNCTION_NAME="deposit(uint256)"
ROOT_FUNCTION_NAME="burnTokens"

# ELock address in ETH
ETH_CONTRACT_ADDRESS=0xe2aC76043137F28e913cd66eD895Ab502f991b8B

# Checker address in GOSH 
CHECKER_ADDRESS=0:bd06195d6975403fa4566f9ad24ed1cd368772f1b0d4c223b2975331b777ed6a

# TOKEN ROOT address in GOSH 
ROOT_ADDRESS=0:30775c35de6c215b378f12274523ba6e77f287ac47c930310d83a8f39be3698b

# Paths to keys, this pubkey should match GOSH config params
VALIDATORS_KEY_PATH=/home/user/GOSH/gosh-proposer/tests/keys.json

# Private key of ETH validator wallet
ETH_PRIVATE_KEY_PATH=/home/user/GOSH/gosh-proposer/tests/eth.private.key
```

8) On each validator run checkers in loop:

```bash
loop:
  deposit-proposal-checker
  withdraw_proposal_checker
  sleep 30 sec
```

9) On ONE! validator  (only one for not to spam with proposals) also run in loop:

```bash
loop:
  gosh_proposer
  withdraw_proposal_checker create
  sleep 30 sec
```

10) Add monitoring for checker's last ETH block if it is too Old decrease sleep time

Get checker's last verified block:

```
NEW_HASH=$(gosh-cli runx --addr $CHECKER_ADDRESS --abi $CHECKER_ABI -m getStatus | jq -r .prevhash)
```

Get block number for hash:

```bash
curl --location --request POST 'https://eth.getblock.io/sepolia/' --header 'x-api-key: 7d0e158c-a55e-46dc-9ca3-ef7586215225' --header 'Content-Type: application/json' --data-raw '{"jsonrpc": "2.0","method":"eth_getBlockByHash","params": ["0x38cd31a32f195ce34bc35ddb5c6dab11188aa6fc5343b9c3017bf909a7a097af", true],"id": "getblock.io"}' | jq .result.number
```

Get latest block number: 

```bash
curl --location --request POST 'https://eth.getblock.io/sepolia/' --header 'x-api-key: 7d0e158c-a55e-46dc-9ca3-ef7586215225' --header 'Content-Type: application/json' --data-raw '{"jsonrpc": "2.0","method":"eth_getBlockByNumber","params": ["latest", true],"id": "getblock.io"}' | jq .result.number
```

if latest block num is too far from the latest sleep time can be reduced in both loops (7 and 8) to catch up