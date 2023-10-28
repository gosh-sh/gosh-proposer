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
USER_KEY = "owner.keys.json"

GOSH_CLI = os.environ.get('GOSH_CLI', 'gosh-cli')

trace_cmd = True
WAS_ERROR = False


GOSH_GIVER_ADDRESS = "-1:9999999999999999999999999999999999999999999999999999999999999999"
# WEENUS_ERC20_ROOT = "0x7439E9Bb6D8a84dd3A23fe621A30F95403F87fB9"
# XEENUS_ERC20_ROOT = "0xc21d97673B9E0B3AA53a06439F71fDc1facE393B"
ERC20_ROOTS = {
    "WEENUS": {
        "name": "Weenus 💪",
        "symbol": "WEENUS",
        "decimals": "18",
        "ethroot": "0x7439E9Bb6D8a84dd3A23fe621A30F95403F87fB9"
    },
    "XEENUS": {
        "name": "Xeenus 💪",
        "symbol": "XEENUS",
        "decimals": "18",
        "ethroot": "0xc21d97673B9E0B3AA53a06439F71fDc1facE393B"
    },
    "GETH": {
        "name": "geth",
        "symbol": "gth",
        "decimals": "18",
        "ethroot": "0x0000000000000000000000000000000000000000"
    }
}
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
            print(f"Command `{command}` execution failed: {output} {e.stderr}")
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


def make_eth_deposit(elock_address, main_pubkey, value):
    # Deposit Ether
    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} "deposit(uint256)" {main_pubkey} \
--private-key $ETH_PRIVATE_KEY --value {value}''', '../contracts/l1/')


def make_erc20_deposit(elock_address, main_pubkey, token_name="WEENUS", value=1_000_000_000_000_000_000):
    # Deposit ERC-20
    erc20_root = ERC20_ROOTS.get(token_name)
    if erc20_root is None:
        print("Wrong token name")
        exit(1)
    erc20_root = erc20_root["ethroot"]
    execute_cmd(f'''cast send --rpc-url $ETH_URL {erc20_root} "approve(address,uint256)" {elock_address} \
{value} --private-key $ETH_PRIVATE_KEY''')
    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} "depositERC20(address,uint256,uint256)" {erc20_root} \
 {value} {main_pubkey} --private-key $ETH_PRIVATE_KEY''')


def make_erc20_withdrawal(elock_address, token_name):
    erc20_root = ERC20_ROOTS.get(token_name)
    if erc20_root is None:
        print("Wrong token name")
        exit(1)
    erc20_root = erc20_root["ethroot"]
    output = execute_cmd(f'cast call --rpc-url $ETH_URL {elock_address} "getERC20Approvement(address,address)" \
{erc20_root} $ETH_WALLET_ADDR')
    print("erc20 approvement: ", output)
    (value, commission) = (int(output[2:66], 16), int(output[66:], 16))
    print(f'erc20 approvement decoded: {value=} {commission=}')

    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} "withdrawERC20(address)" {erc20_root} \
--private-key $ETH_PRIVATE_KEY --value {commission}''', '../contracts/l1/')


def deploy_gosh_contract(tvc_path: str, key_path: str, constructor_args: str, abi_path: str = None,
                         balance: int = 100_000_000_000):
    tmp_tvc_path = f"{tvc_path}_tmp"
    if abi_path is None:
        abi_path = tvc_path.replace("tvc", "abi.json")
    execute_cmd(f'cp {tvc_path} {tmp_tvc_path}')
    address = execute_cmd(f'''gosh-cli -j genaddr --save --abi {abi_path} \
--setkey {key_path} {tmp_tvc_path} | jq .raw_address | cut -d '"' -f 2''')
    execute_cmd(f'gosh-cli -j callx --addr {GOSH_GIVER_ADDRESS} \
--abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value {balance} --bounce false \
--allBalance false --payload ""  --dest {address}')
    execute_cmd(f'gosh-cli -j deployx --abi {abi_path} --keys {key_path} \
{tmp_tvc_path} {constructor_args}')
    execute_cmd(f"rm {tmp_tvc_path}")
    return address


def deploy_glock(last_blocks):
    # Deploy receiver and checker contracts
    receiver_address = deploy_gosh_contract("../contracts/l2/receiver.tvc", MAIN_KEY, '')
    checker_address = deploy_gosh_contract("../contracts/l2/checker.tvc", MAIN_KEY, f'''\
--prevhash {last_blocks["eth"]["hash"]} --receiver {receiver_address} ''', balance=10_000_000_000_000)

    # Set checker ready
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setReady --ready true''')

    # set proposal code in checker
    proposal_code = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/proposal_test.tvc \
| jq .code | cut -d '"' -f 2''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setProposalCode --code {proposal_code}''')

    # set root code in checker and receiver
    root_code = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/RootTokenContract.tvc \
| jq .code | cut -d '"' -f 2''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setRootCode --code {root_code}''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/receiver.abi.json --keys {MAIN_KEY} \
--addr {receiver_address} -m setRootCode --code {root_code}''')

    # Set wallet code in checker
    code_wallet = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/TONTokenWallet.tvc \
| jq .code | cut -d '"' -f 2''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setWalletCode --code {code_wallet}''')

    # Set index code in checker
    code_index = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/indexwallet.tvc \
| jq .code | cut -d '"' -f 2''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setIndexWalletCode --code {code_index}''')

    return checker_address


def deploy_glock_root(token_name, checker_address):
    root_data_orig = ERC20_ROOTS.get(token_name)
    if root_data_orig is None:
        print("Wrong token name")
        exit(1)
    root_data = root_data_orig.copy()
    eth_root = root_data["ethroot"][2:]
    root_data["ethroot"] = f"0x000000000000000000000000{eth_root}"
    params = json.dumps(root_data)

    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m deployRootContract '{{"root":{params}}}' ''')

    root_address = execute_cmd(f'''gosh-cli runx --abi ../contracts/l2/checker.abi.json \
--addr {checker_address} -m getRootAddr '{{"data":{params}}}' | jq -r .value0''')
    return root_address


def parse_events(elock_address):
    events = execute_cmd(f'''ETH_CONTRACT_ADDRESS={elock_address} withdraw_proposal_checker events''')
    events = json.loads(events)

    deposit_cnt = 0
    withdrawal_cnt = 0
    for event in events:
        if event["name"] == "Deposited":
            deposit_cnt += 1
        if event["name"] == "Withdrawal":
            withdrawal_cnt += 1
    print(f"{deposit_cnt=} {withdrawal_cnt=}")
    if deposit_cnt != 3 or withdrawal_cnt != 3:
        print("Wrong events count")
        exit(1)


def check_index(checker_address, root_data_orig, pubkey):
    root_data = root_data_orig.copy()
    eth_root = root_data["ethroot"][2:]
    root_data["ethroot"] = f"0x000000000000000000000000{eth_root}"
    params = json.dumps(root_data)
    index_address = execute_cmd(f'''gosh-cli runx --abi ../contracts/l2/checker.abi.json \
--addr {checker_address} -m getIndexWalletAddr '{{"data":{params},"pubkey":"{pubkey}"}}' | jq -r .value0''')
    print(f"{index_address=}")
    account = execute_cmd(f'''gosh-cli account {index_address}''')
    convert = json.loads(account)
    return bool(convert)


def get_telemetry(checker_address, elock_address):
    telemetry = execute_cmd(f"CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} l2-telemetry", '../')
    data = json.loads(telemetry)
    print(json.dumps(data, indent=2))


def test_main():
    execute_cmd(f"gosh-cli genphrase --dump {MAIN_KEY}")
    main_pubkey = load_pubkey(MAIN_KEY)
    execute_cmd(f"gosh-cli genphrase --dump {USER_KEY}")
    user_pubkey = load_pubkey(USER_KEY)

    last_blocks = get_last_blocks()
    # elock_address = '0xe3660E9BA4ed1e77f7ABe9D0a83d0B09C835C220'
    elock_address = deploy_elock(last_blocks)
    make_eth_deposit(elock_address, user_pubkey, ELOCK_DEPOSIT_VALUE)
    make_erc20_deposit(elock_address, user_pubkey)
    make_erc20_deposit(elock_address, user_pubkey, token_name="XEENUS")

    execute_cmd('gosh-cli config --is_json true -e $GOSH_URL')

    checker_address = deploy_glock(last_blocks)
    geth_root = deploy_glock_root("GETH", checker_address)
    weenus_root = deploy_glock_root("WEENUS", checker_address)
    # checker_address = "0:85fd115cadd8a4e91d387a4e546ca44576400a6d0fe8085ac659c7f8c748454d"
    get_telemetry(checker_address, elock_address)

    root_data = ERC20_ROOTS.get("XEENUS")
    if root_data is None:
        print("Wrong token name")
        exit(1)
    params = json.dumps(root_data)
    xeenus_root = execute_cmd(f'''gosh-cli runx --abi ../contracts/l2/checker.abi.json \
--addr {checker_address} -m getRootAddr '{{"data":{params}}}' | jq -r .value0''', ignore_error=True)


    root_addresses = [geth_root, weenus_root, xeenus_root]
    time.sleep(20)
    while True:
        print(f"{checker_address=}")
        print(f"{root_addresses=}")

        execute_cmd(f'''MAX_BLOCK_IN_ONE_CHUNK=40 ETH_CONTRACT_ADDRESS={elock_address} \
CHECKER_ADDRESS={checker_address} gosh_proposer''', '../', ignore_error=True)

        if not WAS_ERROR:
            prop_address = execute_cmd(f'''gosh-cli runx --addr {checker_address} \
--abi ../contracts/l2/checker.abi.json -m getAllProposalAddr''')
            prop_address = json.loads(prop_address)['value0']
            print(f"{prop_address=}")
            if len(prop_address) != 0:
                prop_address = prop_address[-1]
                execute_cmd(f'''gosh-cli -j callx --addr {prop_address} --abi ../contracts/l2/proposal_test.abi.json  \
-m setvdict --key {main_pubkey}''')
                get_telemetry(checker_address, elock_address)
                execute_cmd(f'''VALIDATORS_KEY_PATH=tests/{MAIN_KEY} ETH_CONTRACT_ADDRESS={elock_address} \
CHECKER_ADDRESS={checker_address} deposit-proposal-checker''', '../')

        time.sleep(60)
        index_found = True
        for root_data in ERC20_ROOTS:
            index_found = index_found and check_index(checker_address, ERC20_ROOTS[root_data], user_pubkey)
        if not index_found:
            continue
        print("All indexes exist")
        break

    root_cnt = len(root_addresses)
    token_wallets = []
    for root in root_addresses:
        token_wallet = execute_cmd(f'''gosh-cli runx --addr {root} \
--abi ../contracts/l2/RootTokenContract.abi -m getWalletAddress --owner null --pubkey {user_pubkey} \
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

    if root_cnt != 0:
        print(f"Wrong wallets cnt {root_cnt} != 0")
        exit(1)

    print(f'token_wallets = {token_wallets}')

    for wallet in token_wallets:
        t_wallet = wallet["wallet"]
        balance = wallet["balance"]
        execute_cmd(f'''gosh-cli callx --addr {t_wallet} --abi ../contracts/l2/TONTokenWallet.abi --keys {USER_KEY} \
-m burnTokens --_answer_id 0 --to $ETH_WALLET_ADDR --tokens {balance}''')
    time.sleep(20)
    get_telemetry(checker_address, elock_address)
    find_burns = execute_cmd(f'''CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker find_burns''')
    print(f'{find_burns=}')
    execute_cmd(f'''CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker create''')

    time.sleep(10)
    get_telemetry(checker_address, elock_address)

    execute_cmd(f'''CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker''')

    time.sleep(60)

    make_erc20_withdrawal(elock_address, "WEENUS")
    make_erc20_withdrawal(elock_address, "XEENUS")
    parse_events(elock_address)

    get_telemetry(checker_address, elock_address)


test_main()
print("Success!!!")
