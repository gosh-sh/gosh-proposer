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
    last_blocks = execute_cmd("cargo run -p withdraw_proposal_checker --release  -- get_last_blocks", "..")
    print(f"last blocks out: {last_blocks}")
    res = json.loads(last_blocks)
    return res


def test_main():
    execute_cmd(f"gosh-cli genphrase --dump {MAIN_KEY}")
    main_pubkey = load_pubkey(MAIN_KEY)
    last_blocks = get_last_blocks()

    elock_address = '0xffC9184493F3DF3428E8B989981f33228aCa7aEE'
    print(f"{elock_address=}")

    elock_address = execute_cmd(f'''forge create --rpc-url $ETH_URL --private-key $ETH_PRIVATE_KEY src/Elock.sol:Elock --constructor-args {last_blocks["gosh"]["id"]} $ETH_WALLET_ADDR [$ETH_WALLET_ADDR] | grep "Deployed to: " | cut -d ' ' -f 3''', '../contracts/l1/')
    elock_address = elock_address.strip()
    print(f"{elock_address=}")
    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} "deposit(uint256)" {main_pubkey} --private-key $ETH_PRIVATE_KEY --value 0.02ether''', '../contracts/l1/')
    execute_cmd(f'''cast send --rpc-url $ETH_URL {elock_address} --private-key $ETH_PRIVATE_KEY --value 0.01ether''', '../contracts/l1/')

    execute_cmd('gosh-cli config --is_json true -e $GOSH_URL')

    execute_cmd('cp ../contracts/l2/checker.tvc ../contracts/l2/checker2.tvc')
    checker_address = execute_cmd(f'''gosh-cli -j genaddr --save --abi ../contracts/l2/checker.abi.json --setkey keys.json ../contracts/l2/checker2.tvc | jq .raw_address | cut -d '"' -f 2''')
    print(f"{checker_address=}")
    execute_cmd(f'gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 100000000000 --bounce false --allBalance false --payload ""  --dest {checker_address}')
    execute_cmd(f'gosh-cli -j deployx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} ../contracts/l2/checker2.tvc --prevhash {last_blocks["eth"]["hash"]}')
    execute_cmd("rm ../contracts/l2/checker2.tvc")

    proposal_code = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/proposal_test.tvc | jq .code | cut -d '"' -f 2''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} --addr {checker_address} -m setProposalCode --code {proposal_code}''')

    execute_cmd(f"gosh-cli genphrase --dump {ROOT_KEY}")
    root_pubkey = load_pubkey(ROOT_KEY)
    execute_cmd('cp ../contracts/l2/RootTokenContract.tvc ../contracts/l2/RootTokenContract2.tvc')
    root_address = execute_cmd(f'''gosh-cli -j genaddr --save --abi ../contracts/l2/RootTokenContract.abi --setkey {ROOT_KEY} ../contracts/l2/RootTokenContract2.tvc | jq .raw_address | cut -d '"' -f 2''')
    print("{root_address=}")
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} --addr {checker_address} -m setRootContract --root {root_address}''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/checker.abi.json --keys {MAIN_KEY} --addr {checker_address} -m setReadyRoot --ready true''')

    code_wallet = execute_cmd('''gosh-cli -j decode stateinit --tvc ../contracts/l2/TONTokenWallet.tvc | jq .code | cut -d '"' -f 2''')

    execute_cmd(f'''gosh-cli -j callx --addr -1:9999999999999999999999999999999999999999999999999999999999999999 --abi SetcodeMultisigWallet.abi.json --keys devgiver9.json -m submitTransaction --value 1000000000000 --bounce false --allBalance false --payload ""  --dest {root_address}''')
    execute_cmd(f'''gosh-cli -j deployx --abi ../contracts/l2/RootTokenContract.abi --keys {ROOT_KEY} ../contracts/l2/RootTokenContract2.tvc --name "geth" --symbol "gth" --decimals 18 --root_pubkey {root_pubkey} --root_owner null --total_supply 0 --checker {checker_address} --oldroot_ null --newroot_ null''')
    execute_cmd(f'''gosh-cli -j callx --abi ../contracts/l2/RootTokenContract.abi --keys {ROOT_KEY} --addr {root_address} -m setWalletCode --wallet_code {code_wallet} --_answer_id 0''')
    execute_cmd('rm ../contracts/l2/RootTokenContract2.tvc')

    while(True):
        print(f"{checker_address=}")
        print(f"{root_address=}")

        time.sleep(20)
        execute_cmd(f'''MAX_BLOCK_IN_ONE_CHUNK=40 ETH_CONTRACT_ADDRESS={elock_address} CHECKER_ADDRESS={checker_address} make run_proposer''', '../', ignore_error=True)
        if WAS_ERROR:
            continue

        prop_address = execute_cmd(f'''gosh-cli runx --addr {checker_address} --abi ../contracts/l2/checker.abi.json -m getAllProposalAddr''')
        prop_address = json.loads(prop_address)['value0']
        print(f"{prop_address=}")
        if len(prop_address) == 0:
            continue
        prop_address = prop_address[-1]
        execute_cmd(f'''gosh-cli -j callx --addr {prop_address} --abi ../contracts/l2/proposal_test.abi.json  -m setvdict --key {main_pubkey}''')

        execute_cmd(f'''VALIDATORS_KEY_PATH=tests/{MAIN_KEY} ETH_CONTRACT_ADDRESS={elock_address} CHECKER_ADDRESS={checker_address} make run_deposit''', '../')

        token_wallet = execute_cmd(f'''gosh-cli runx --addr {root_address} --abi ../contracts/l2/RootTokenContract.abi -m getWalletAddress --owner null --pubkey {main_pubkey} | jq -r .value0''', ignore_error=True)
        print(f"{token_wallet=}")
        if WAS_ERROR:
            continue

        TOKEN_BALANCE = execute_cmd(f'''gosh-cli runx --addr {token_wallet} --abi ../contracts/l2/TONTokenWallet.abi -m getDetails| jq -r .balance''', ignore_error=True)
        print(f"{TOKEN_BALANCE=}")
        if WAS_ERROR:
            continue
        if int(TOKEN_BALANCE) > 0:
            break

    token_wallet = '0:574d2243756b01fd8af4b12e6bc2645a0a1148035636228ae5dfe289355e29e9'
    TOKEN_BALANCE = '19980000000000000'
    root_address = '0:4ee932bde06753af9d1ec4dd63663775274b20c4125618cd3f4572f09e227b5b'
    elock_address = '0xF42b3DA39B4b52473E6779C4545740135Ffea03D'


    execute_cmd(f'''gosh-cli callx --addr {token_wallet} --abi ../contracts/l2/TONTokenWallet.abi --keys {MAIN_KEY} -m burnTokens --_answer_id 0 --to $ETH_WALLET_ADDR --tokens {TOKEN_BALANCE}''')

    time.sleep(10)

    find_burns = execute_cmd(f'''ROOT_ADDRESS={root_address} ETH_CONTRACT_ADDRESS={elock_address} cargo run -p withdraw_proposal_checker --release  -- find_burns''')
    print(f'{find_burns=}')
    execute_cmd(f'''ROOT_ADDRESS={root_address} ETH_CONTRACT_ADDRESS={elock_address} cargo run -p withdraw_proposal_checker --release  -- create''')

    time.sleep(10)

    execute_cmd(f'''ROOT_ADDRESS={root_address} ETH_CONTRACT_ADDRESS={elock_address} make run_withdraw''', "../")


test_main()
