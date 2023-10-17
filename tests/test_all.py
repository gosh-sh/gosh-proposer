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
ROOT_KEY = "root.keys.json"

GOSH_CLI = os.environ.get('GOSH_CLI', 'gosh-cli')


trace_cmd = True
WAS_ERROR = False


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


def deploy_elock(last_blocks, main_pubkey):
    elock_address = execute_cmd(f'''\
forge create --rpc-url $ETH_URL --private-key $ETH_PRIVATE_KEY src/Elock.sol:Elock --constructor-args \
{last_blocks["gosh"]["id"]} $ETH_WALLET_ADDR [$ETH_WALLET_ADDR] \
| grep "Deployed to: " \
| cut -d ' ' -f 3''', '../contracts/l1/')
    elock_address = elock_address.strip()
    print(f"{elock_address=}")

    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} "deposit(uint256)" {main_pubkey} \
--private-key $ETH_PRIVATE_KEY --value {ELOCK_DEPOSIT_VALUE}''', '../contracts/l1/')
    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} --private-key $ETH_PRIVATE_KEY \
--value {ELOCK_INIT_VALUE}''', '../contracts/l1/')
    return elock_address


def deploy_glock(last_blocks, root_pubkey):
    execute_cmd('cp ../contracts/l2/checker.tvc ../contracts/l2/checker2.tvc')
    checker_address = execute_cmd(f'''gosh-cli -j genaddr --save --abi ../contracts/l2/checker.abi.json \
--setkey keys.json ../contracts/l2/checker2.tvc | jq .raw_address | cut -d '"' -f 2''')
    print(f"{checker_address=}")
    execute_cmd(f'gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 \
--abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 100000000000 --bounce false \
--allBalance false --payload ""  --dest {checker_address}')
    execute_cmd(f'gosh-cli -j deployx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
../contracts/l2/checker2.tvc --prevhash {last_blocks["eth"]["hash"]}')
    execute_cmd("rm ../contracts/l2/checker2.tvc")

    proposal_code = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/proposal_test.tvc \
| jq .code | cut -d '"' -f 2''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setProposalCode --code {proposal_code}''')

    root_code = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/RootTokenContract.tvc \
| jq .code | cut -d '"' -f 2''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker0.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setRootCode --code {root_code}''')

    # root_params = {
    #     "name": "WEENUS",
    #     "symbol": "WNS",
    #     "decimals": "18",
    #     "ethroot": "0x0000000000000000000000007439E9Bb6D8a84dd3A23fe621A30F95403F87fB9"
    # }
    root_params = {
        "name": "geth",
        "symbol": "gth",
        "decimals": "18",
        "ethroot": ""
    }
    params = json.dumps(root_params)

    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker0.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m deployRootContract '{params}' ''')

    root_address = execute_cmd(f'''gosh-cli runx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m getRootAddr '{{"data":{params}}}' \
| jq -r .value0''', ignore_error=True)

    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} \
--addr {checker_address} -m setReadyRoot --ready true''')

    code_wallet = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/TONTokenWallet.tvc \
| jq .code | cut -d '"' -f 2''')

    execute_cmd(f'''gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 \
--abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 1000000000000 --bounce false \
--allBalance false --payload ""  --dest {root_address}''')

    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/RootTokenContract.abi --keys {ROOT_KEY} \
--addr {root_address} -m setWalletCode --wallet_code {code_wallet} --_answer_id 0''')

    return checker_address, root_address


def test_main():
    execute_cmd(f"gosh-cli genphrase --dump {MAIN_KEY}")
    main_pubkey = load_pubkey(MAIN_KEY)
    last_blocks = get_last_blocks()
    elock_address = deploy_elock(last_blocks, main_pubkey)

    execute_cmd(f"gosh-cli genphrase --dump {ROOT_KEY}")
    root_pubkey = load_pubkey(ROOT_KEY)

    execute_cmd('gosh-cli config --is_json true -e $GOSH_URL')

    (checker_address, root_address) = deploy_glock(last_blocks, root_pubkey)

    while True:
        print(f"{checker_address=}")
        print(f"{root_address=}")

        time.sleep(20)
        execute_cmd(f'''MAX_BLOCK_IN_ONE_CHUNK=40 ETH_CONTRACT_ADDRESS={elock_address} \
CHECKER_ADDRESS={checker_address} gosh_proposer''', '../', ignore_error=True)
        if not WAS_ERROR:
            prop_address = execute_cmd(f'''gosh-cli runx --addr {checker_address} \
--abi ../contracts/l2/checker.abi.json -m getAllProposalAddr''')
            prop_address = json.loads(prop_address)['value0']
            print(f"{prop_address=}")
            if len(prop_address) == 0:
                continue
            prop_address = prop_address[-1]
            execute_cmd(f'''gosh-cli -j callx --addr {prop_address} --abi ../contracts/l2/proposal_test.abi.json  \
-m setvdict --key {main_pubkey}''')

            execute_cmd(f'''VALIDATORS_KEY_PATH=tests/{MAIN_KEY} ETH_CONTRACT_ADDRESS={elock_address} \
CHECKER_ADDRESS={checker_address} deposit-proposal-checker''', '../')

        token_wallet = execute_cmd(f'''gosh-cli runx --addr {root_address} \
--abi ../contracts/l2/RootTokenContract.abi -m getWalletAddress --owner null --pubkey {main_pubkey} \
| jq -r .value0''', ignore_error=True)
        print(f"{token_wallet=}")
        if WAS_ERROR:
            continue

        TOKEN_BALANCE = execute_cmd(f'''gosh-cli runx --addr {token_wallet} \
--abi ../contracts/l2/TONTokenWallet.abi -m getDetails| jq -r .balance''', ignore_error=True)
        print(f"{TOKEN_BALANCE=}")
        if WAS_ERROR:
            continue
        if int(TOKEN_BALANCE) > 0:
            break

    execute_cmd(f'''gosh-cli callx --addr {token_wallet} --abi ../contracts/l2/TONTokenWallet.abi --keys {MAIN_KEY} \
-m burnTokens --_answer_id 0 --to $ETH_WALLET_ADDR --tokens {TOKEN_BALANCE}''')

    time.sleep(10)

    find_burns = execute_cmd(f'''ROOT_ADDRESS={root_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker find_burns''')
    print(f'{find_burns=}')
    execute_cmd(f'''ROOT_ADDRESS={root_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker create''')

    time.sleep(10)

    execute_cmd(f'''ROOT_ADDRESS={root_address} ETH_CONTRACT_ADDRESS={elock_address} \
withdraw_proposal_checker''', "../")


test_main()
