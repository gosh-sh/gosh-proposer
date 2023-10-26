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
--addr {checker_address} -m deployRootContract '{params}' ''')

    root_address = execute_cmd(f'''gosh-cli runx --abi ../contracts/l2/checker.abi.json \
--addr {checker_address} -m getRootAddr '{{"data":{params}}}' | jq -r .value0''', ignore_error=True)
    return root_address

#         # TODO: check that checker gives 1000 evers after deploy
#         execute_cmd(f'''gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 \
# --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 1000000000000 --bounce false \
# --allBalance false --payload ""  --dest {root_address}''')

#         if root_params == roots[1]:
#             execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/RootTokenContract.abi --keys {MAIN_KEY} \
# --addr {root_address} -m setWalletCode --wallet_code {code_wallet} --_answer_id 0''')

        # root_addresses.append(root_address)

    # return checker_address, root_addresses


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


def test_main():
    execute_cmd(f"gosh-cli genphrase --dump {MAIN_KEY}")
    main_pubkey = load_pubkey(MAIN_KEY)
    last_blocks = get_last_blocks()
    elock_address = deploy_elock(last_blocks)
    # elock_address = '0x69736886754698f0B7335B8b7505C6A169D78a5f'
    make_eth_deposit(elock_address, main_pubkey, ELOCK_DEPOSIT_VALUE)
    make_erc20_deposit(elock_address, main_pubkey)
    make_erc20_deposit(elock_address, main_pubkey, token_name="XEENUS")

    execute_cmd('gosh-cli config --is_json true -e $GOSH_URL')

    checker_address = deploy_glock(last_blocks)
    # checker_address = "0:3ce17532c00eaec23640948c243f6612f47cbbd13d3303b5396ec2716bbb7a15"
    geth_root = deploy_glock_root("GETH", checker_address)
    weenus_root = deploy_glock_root("WEENUS", checker_address)

    root_data = ERC20_ROOTS.get("XEENUS")
    if root_data is None:
        print("Wrong token name")
        exit(1)
    params = json.dumps(root_data)
    xeenus_root = execute_cmd(f'''gosh-cli runx --abi ../contracts/l2/checker.abi.json \
    --addr {checker_address} -m getRootAddr '{{"data":{params}}}' | jq -r .value0''', ignore_error=True)

    # checker_address = '0:ab3436466ffd5c7516f00758ab94ee155e6994115b69f3337c736a813b30a556'
    # root_addresses = ['0:0fce959e8f3a408ab2c5867dc5a2e2c3513c075a28d46f993e2d341cdec9a5a7',
    #                   '0:c33ce162dbca12d2317722ddc7e81a4c2237b319a8c4e5370b89769ae229b0b1']

    root_addresses = [geth_root, weenus_root, xeenus_root]
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

    time.sleep(10)

    make_erc20_withdrawal(elock_address, "WEENUS")
    make_erc20_withdrawal(elock_address, "XEENUS")
    parse_events(elock_address)


test_main()

