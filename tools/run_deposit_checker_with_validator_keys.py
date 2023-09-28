import json
import os
import subprocess
import base64

# This script reads keys from validator config and tries to run 
# deposit_proposal_checker with each of them.


# path to the validators config
CONFIG_PATH = os.environ.get('VALIDATOR_CONFIG_PATH', '/opt/ton/ton-node/configs/config.json')
CHECKER_ADDRESS = os.environ.get('CHECKER_ADDRESS')
GOSH_CLI = os.environ.get('GOSH_CLI_PATH', 'gosh-cli')
VALIDATORS_KEY_PATH = os.environ.get('VALIDATORS_KEY_PATH', 'keys.json')
DEPOSIT_PROPOSAL_CHECKER = os.environ.get('DEPOSIT_PROPOSAL_CHECKER', 'deposit-proposal-checker')


CHECKER_ABI = '''
{
    "ABI version": 2,
    "version": "2.3",
    "header": ["pubkey", "time", "expire"],
    "functions": [
        {
            "name": "getAllProposalAddr",
            "inputs": [
            ],
            "outputs": [
                {"name":"value0","type":"address[]"}
            ]
        }
    ],
    "data": [
    ],
    "events": [
    ],
    "fields": [
    ]
}
'''


def execute_cmd(cmd: str):
    print(cmd)
    try:
        output = subprocess.check_output(cmd, shell=True).decode("utf-8")
    except subprocess.CalledProcessError as e:
        output = e.output.decode("utf-8")
        print("Error occurred: ", output)

    return output


def main():
    if CHECKER_ADDRESS == "":
        print("Failed to get CHECKER_ADDRESS")
        exit(1)

    # load data
    with open(CONFIG_PATH) as f:
        data = f.read()
    mapping = json.loads(data)

    for key_id in mapping["validator_key_ring"]:
        key_ring = mapping["validator_key_ring"][key_id]
        private_key = base64.b64decode(key_ring["pvt_key"]).hex()
        execute_cmd(f"{GOSH_CLI} getkeypair -p {private_key} -o {VALIDATORS_KEY_PATH}")
        execute_cmd(f"VALIDATORS_KEY_PATH={VALIDATORS_KEY_PATH} {DEPOSIT_PROPOSAL_CHECKER}")

main()



