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
    telemetry = execute_cmd(f"CHECKER_ADDRESS={checker_address} ETH_CONTRACT_ADDRESS={elock_address} l2-telemetry", '../')
    data = json.loads(telemetry)
    print(json.dumps(data, indent=2))


def main():
    elock_address = '0x3FE9D27909cf54A492C35BC3a2920b5573aA422d'
    checker_address = '0:86ce56545db94970c6cce080f91ddac23ecb2df33ac29bf3935f1b4cd648984d'
    main_pubkey = load_pubkey(MAIN_KEY)
    while True:
        time.sleep(60)
        get_telemetry(checker_address, elock_address)
        execute_cmd(f'''MAX_BLOCK_IN_ONE_CHUNK=40 CHECKER_ADDRESS={checker_address} \
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


main()

