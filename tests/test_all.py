import os
import subprocess
import json
import time


ETH_WALLET_ADDR = os.environ.get('ETH_WALLET_ADDR')
os.environ["TEST_TRACE"] = "/home/user/GOSH/gosh-proposer/tests/trace.log"
os.environ["ETH_URL"] = "https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a"
os.environ["GOSH_URL"] = "https://sh.network.gosh.sh"
os.environ["ETH_VALIDATOR_CONTRACT_ADDRESS"] = ETH_WALLET_ADDR
os.environ["MAX_BLOCK_IN_ONE_CHUNK"] = "40"
MAIN_KEY = "keys.json"

GOSH_CLI = os.environ.get('GOSH_CLI', 'gosh-cli')

trace_cmd = True
WAS_ERROR = False


GIVER_ADDRESS = "-1:9999999999999999999999999999999999999999999999999999999999999999"
ERC20_ROOT = "0x7439E9Bb6D8a84dd3A23fe621A30F95403F87fB9"
ELOCK_DEPOSIT_VALUE = '0.02ether'
ELOCK_INIT_VALUE = '0.01ether'


def execute_cmd(command: str, work_dir=".", ignore_error=False):
    global WAS_ERROR
    command = f"cd {work_dir} && {command}"
    WAS_ERROR = False
    if trace_cmd:
        print(command)
    try:
        output = subprocess.check_output(command, shell=True).decode("utf-8")
    except subprocess.CalledProcessError as e:
        output = e.output.decode("utf-8")
        WAS_ERROR = True
        if not ignore_error:
            print(f"Command `{command}` execution failed: {output}")
            exit(1)

    return output.strip()


def load_pubkey(path):
    with open(path) as f:
        data = f.read()
    mapping = json.loads(data)
    public = f"0x{mapping['public']}"
    print(f"public of {path}: {public}")
    return public


def get_last_blocks():
    last_blocks = execute_cmd("withdraw_proposal_checker get_last_blocks", "..")
    print(f"last blocks out: {last_blocks}")
    res = json.loads(last_blocks)
    return res


def deploy_elock(last_blocks):
    # Deploy ELock contract
    elock_address = execute_cmd(f'''\
forge create --rpc-url $ETH_URL --private-key $ETH_PRIVATE_KEY src/Elock.sol:Elock --constructor-args \
{last_blocks["gosh"]["id"]} $ETH_WALLET_ADDR [$ETH_WALLET_ADDR] \
| grep "Deployed to: " \
| cut -d ' ' -f 3''', '../contracts/l1/')
    elock_address = elock_address.strip()
    print(f"{elock_address=}")

    # Send Operational balance for ELock
    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} --private-key $ETH_PRIVATE_KEY \
--value {ELOCK_INIT_VALUE}''', '../contracts/l1/')

    print(f"ELock address: {elock_address}")
    return elock_address


def make_elock_deposits(elock_address, main_pubkey):
    # Deposit Ether
    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} "deposit(uint256)" {main_pubkey} \
--private-key $ETH_PRIVATE_KEY --value {ELOCK_DEPOSIT_VALUE}''', '../contracts/l1/')

    # Deposit ERC-20
    execute_cmd(f'''cast send --rpc-url $ETH_URL {ERC20_ROOT} "approve(address,uint256)" {elock_address} \
1000000000000000000 --private-key $ETH_PRIVATE_KEY''')
    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} "depositERC20(address,uint256,uint256)" {ERC20_ROOT} \
 1000000000000000000 {main_pubkey} --private-key $ETH_PRIVATE_KEY''')


def deploy_gosh_contract(tvc_path: str, key_path: str, constructor_args: str, abi_path: str = None):
    tmp_tvc_path = "tmp.tvc"
    if abi_path is None:
        abi_path = tvc_path.replace("tvc", "abi.json")
    execute_cmd(f'cp {tvc_path} {tmp_tvc_path}')
    address = execute_cmd(f'''gosh-cli -j genaddr --save --abi {abi_path} \
--setkey {key_path} {tmp_tvc_path} | jq .raw_address | cut -d '"' -f 2''')
    execute_cmd(f'gosh-cli -j callx --addr {GIVER_ADDRESS} \
--abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 100000000000 --bounce false \
--allBalance false --payload ""  --dest {address}')
    execute_cmd(f'gosh-cli -j deployx --abi {abi_path} --keys {key_path} \
{tmp_tvc_path} {constructor_args}')
    execute_cmd(f"rm {tmp_tvc_path}")
    return address


def deploy_glock(last_blocks):
    receiver_address = deploy_gosh_contract("../contracts/l2/receiver.tvc", MAIN_KEY, '')

    checker_address = deploy_gosh_contract("../contracts/l2/checker.tvc", MAIN_KEY, f'''\
--prevhash {last_blocks["eth"]["hash"]} --receiver {receiver_address} ''')

    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setReady --ready true''')

    proposal_code = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/proposal_test.tvc \
| jq .code | cut -d '"' -f 2''')

    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setProposalCode --code {proposal_code}''')

    root_code = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/RootTokenContract.tvc \
| jq .code | cut -d '"' -f 2''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setRootCode --code {root_code}''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/receiver.abi.json --keys {MAIN_KEY} \
--addr {receiver_address} -m setRootCode --code {root_code}''')

    code_wallet = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/TONTokenWallet.tvc \
| jq .code | cut -d '"' -f 2''')
    roots = [
      {
        "name": "Weenus 💪",
        "symbol": "WEENUS",
        "decimals": "18",
        "ethroot": "0x0000000000000000000000007439e9bb6d8a84dd3a23fe621a30f95403f87fb9"
      }, {
        "name": "geth",
        "symbol": "gth",
        "decimals": "18",
        "ethroot": "0x0000000000000000000000000000000000000000000000000000000000000000"
      }
    ]
    root_addresses = []

    for root_params in roots:
        params = json.dumps(root_params)

        execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m deployRootContract '{params}' ''')

        root_address = execute_cmd(f'''gosh-cli runx --abi ../contracts/l2/checker.abi.json \
--addr {checker_address} -m getRootAddr '{{"data":{params}}}' \
| jq -r .value0''', ignore_error=True)

        # TODO: check that checker gives 1000 evers after deploy
        execute_cmd(f'''gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 \
--abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 1000000000000 --bounce false \
--allBalance false --payload ""  --dest {root_address}''')

        execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/RootTokenContract.abi --keys {MAIN_KEY} \
--addr {root_address} -m setWalletCode --wallet_code {code_wallet} --_answer_id 0''')
        root_addresses.append(root_address)

    return checker_address, root_addresses


def test_main():
    execute_cmd(f"gosh-cli genphrase --dump {MAIN_KEY}")
    main_pubkey = load_pubkey(MAIN_KEY)
    last_blocks = get_last_blocks()
    elock_address = deploy_elock(last_blocks)
    # elock_address = '0x49148998414B131712AA09d7f67235569dC8e856'
    make_elock_deposits(elock_address, main_pubkey)

    execute_cmd('gosh-cli config --is_json true -e $GOSH_URL')

    (checker_address, root_addresses) = deploy_glock(last_blocks)

    # checker_address = '0:ab3436466ffd5c7516f00758ab94ee155e6994115b69f3337c736a813b30a556'
    # root_addresses = ['0:0fce959e8f3a408ab2c5867dc5a2e2c3513c075a28d46f993e2d341cdec9a5a7',
    #                   '0:c33ce162dbca12d2317722ddc7e81a4c2237b319a8c4e5370b89769ae229b0b1']

    while True:
        print(f"{checker_address=}")
        print(f"{root_addresses=}")

        execute_cmd(f'''MAX_BLOCK_IN_ONE_CHUNK=40 ETH_CONTRACT_ADDRESS={elock_address} \
CHECKER_ADDRESS={checker_address} gosh_proposer''', '../', ignore_error=True)
        time.sleep(20)
        if not WAS_ERROR:
            prop_address = execute_cmd(f'''gosh-cli runx --addr {checker_address} \
--abi ../contracts/l2/checker.abi.json -m getAllProposalAddr''')
            prop_address = json.loads(prop_address)['value0']
            print(f"{prop_address=}")
            if len(prop_address) != 0:
                prop_address = prop_address[-1]
                execute_cmd(f'''gosh-cli -j callx --addr {prop_address} --abi ../contracts/l2/proposal_test.abi.json  \
-m setvdict --key {main_pubkey}''')

                execute_cmd(f'''VALIDATORS_KEY_PATH=tests/{MAIN_KEY} ETH_CONTRACT_ADDRESS={elock_address} \
CHECKER_ADDRESS={checker_address} deposit-proposal-checker''', '../')
                time.sleep(20)

        root_cnt = len(root_addresses)
        token_wallets = []
        for root in root_addresses:
            token_wallet = execute_cmd(f'''gosh-cli runx --addr {root} \
--abi ../contracts/l2/RootTokenContract.abi -m getWalletAddress --owner null --pubkey {main_pubkey} \
| jq -r .value0''', ignore_error=True)
            print(f"{token_wallet=}")
            if WAS_ERROR:
                continue

            balance = execute_cmd(f'''gosh-cli runx --addr {token_wallet} \
--abi ../contracts/l2/TONTokenWallet.abi -m getDetails| jq -r .balance''', ignore_error=True)
            print(f"root {root} balance {balance=}")
            if WAS_ERROR:
                continue

            if int(balance) > 0:
                root_cnt -= 1
                token_wallets.append({"root": root, "wallet": token_wallet, "balance": balance})

        if root_cnt == 0:
            break

    print(f'token_wallets = {token_wallets}')

    for wallet in token_wallets:
        t_wallet = wallet["wallet"]
        balance = wallet["balance"]
        execute_cmd(f'''gosh-cli callx --addr {t_wallet} --abi ../contracts/l2/TONTokenWallet.abi --keys {MAIN_KEY} \
-m burnTokens --_answer_id 0 --to $ETH_WALLET_ADDR --tokens {balance}''')

    time.sleep(10)

    find_burns = execute_cmd(f'''CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker find_burns''')
    print(f'{find_burns=}')
    execute_cmd(f'''CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker create''')

    time.sleep(10)

    execute_cmd(f'''CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker''')


test_main()
