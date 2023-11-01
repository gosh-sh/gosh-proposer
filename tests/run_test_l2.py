import os
import subprocess
import json
import time


ETH_WALLET_ADDR = os.environ.get('ETH_WALLET_ADDR')
os.environ["TEST_TRACE"] = "/home/user/GOSH/gosh-proposer/tests/trace.log"
# os.environ["ETH_URL"] = "https://sepolia.infura.io/v3/df557e910fb2496e8d854046cbedb99a"
# os.environ["GOSH_URL"] = "https://sh.network.gosh.sh"
os.environ["ETH_VALIDATOR_CONTRACT_ADDRESS"] = ETH_WALLET_ADDR
os.environ["MAX_BLOCK_IN_ONE_CHUNK"] = "40"
MAIN_KEY = "keys.json"
USER_KEY = "owner.keys.json"

GOSH_CLI = os.environ.get('GOSH_CLI', 'gosh-cli')

trace_cmd = True
WAS_ERROR = False

GOSH_GIVER_ADDRESS = "-1:9999999999999999999999999999999999999999999999999999999999999999"
GIVER_KEY_PATH = 'devgiver9.json'

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


def get_telemetry(checker_address, elock_address):
    telemetry = execute_cmd(f"CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} l2-telemetry")
    data = json.loads(telemetry)
    print(json.dumps(data, indent=2))


def main():
    elock_address = '0x437cAD6B72F80b5721634A6c3dAe2d48cd84EAF8'
    checker_address = '0:410fed0f41e881ace4e4c0eb8e29c629a317375e44d908c46af3f8aafa96bdae'
    root_addresses = ['0:d00dfe58faa74e3e257fa720cbd6408d2d624585117a4f1821a210446c4d6adf', '0:1ee6d908a0dfc3d446a75f69cfc7a4fdfd2aaf01423ea54bf3b55a3b725985c9', '0:a58d1da25512e375195dadc579d66a6e1bd8d99b9834a971299a558eee7af24a']
    main_pubkey = load_pubkey(MAIN_KEY)
    while True:
        # time.sleep(60)
        get_telemetry(checker_address, elock_address)
        execute_cmd(f'''MAX_BLOCK_IN_ONE_CHUNK=64 CHECKER_ADDRESS={checker_address} \
ETH_CONTRACT_ADDRESS={elock_address} gosh_proposer''', ignore_error=True)
        prop_address = execute_cmd(f'''gosh-cli runx --addr {checker_address} \
--abi ../contracts/l2/checker.abi.json -m getAllProposalAddr''')
        prop_address = json.loads(prop_address)['value0']
        print(f"{prop_address=}")
        if len(prop_address) != 0:
            prop_address = prop_address[-1]
            execute_cmd(f'''gosh-cli -j callx --addr {prop_address} --abi ../contracts/l2/proposal_test.abi.json  \
-m setvdict --key {main_pubkey}''')
            execute_cmd(f'''VALIDATORS_KEY_PATH={MAIN_KEY} CHECKER_ADDRESS={checker_address} \
ETH_CONTRACT_ADDRESS={elock_address} deposit-proposal-checker''', ignore_error=True)
        execute_cmd(f'''CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker create''')
        execute_cmd(f'''CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker''')


def generate_contract_addresses(tvc_path: str, key_path: str):
    abi_path = tvc_path.replace("tvc", "abi.json")
    address = execute_cmd(f'''gosh-cli -j genaddr --save --abi {abi_path} --setkey {key_path} {tvc_path} \
| jq .raw_address | cut -d '"' -f 2''')
    return address


def ask_test_giver(address: str, value: int):
    execute_cmd(f'gosh-cli -j callx --addr {GOSH_GIVER_ADDRESS} \
--abi SetcodeMultisigWallet.abi.json --keys {GIVER_KEY_PATH} -m submitTransaction --value {value} --bounce false \
--allBalance false --payload ""  --dest {address}')


def deploy_contract(tvc_path: str, key_path: str, constructor_args: str):
    abi_path = tvc_path.replace("tvc", "abi.json")
    execute_cmd(f'gosh-cli -j deployx --abi {abi_path} --keys {key_path} \
{tvc_path} {constructor_args}')


def get_last_blocks():
    last_blocks = execute_cmd("withdraw_proposal_checker get_last_blocks")
    print(f"last blocks out: {last_blocks}")
    res = json.loads(last_blocks)
    return res


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


def deploy_l2():
    os.environ["GOSH_URL"] = "https://network.gosh.sh"
    os.environ["ETH_URL"] = "https://mainnet.infura.io/v3/df557e910fb2496e8d854046cbedb99a"
    os.environ["GOSH_ENDPOINTS"] = "https://network.gosh.sh"
    os.environ["ETH_NETWORK"] = "wss://mainnet.infura.io/ws/v3/df557e910fb2496e8d854046cbedb99a"
    execute_cmd('gosh-cli config --is_json true -e $GOSH_URL')

    # execute_cmd(f"gosh-cli genphrase --dump {MAIN_KEY}")
    last_blocks = get_last_blocks()

    receiver_tvc = '../contracts/l2/receiver.tvc'
    checker_tvc = '../contracts/l2/checker.tvc'

    receiver_address = generate_contract_addresses(receiver_tvc, MAIN_KEY)
    checker_address = generate_contract_addresses(checker_tvc, MAIN_KEY)

    print(f"{receiver_address=}\n{checker_address=}")
    exit(0)

    # for main change to real one !!!
    # proposal_tvc = '../contracts/l2/proposal_test.tvc'
    proposal_tvc = '../contracts/l2/proposal.tvc'

    # for main change to real one !!!
    ask_test_giver(receiver_address, 100_000_000_000)
    ask_test_giver(checker_address, 10_000_000_000_000)

    deploy_contract(receiver_tvc, MAIN_KEY, '')
    deploy_contract(checker_tvc, MAIN_KEY,
                    f'''--prevhash {last_blocks["eth"]["hash"]} --receiver {receiver_address} ''')

    # Set checker ready
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setReady --ready true''')

    # set proposal code in checker
    proposal_code = execute_cmd(f'''gosh-cli -j decode stateinit --tvc {proposal_tvc} \
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

    print(f"{receiver_address=}\n{checker_address=}")


deploy_l2()
# main()
