import json

INPUT_FILE = 'contracts/l1/out/Elock.sol/Elock.json'
OUTPUT_ABI = 'resources/elock.abi.json'
OUTPUT_IDS = 'resources/identifiers.json'

with open(INPUT_FILE) as f:
    data = f.read()

mapping = json.loads(data)

with open(OUTPUT_ABI, 'w') as abi_file:
    abi_file.write(json.dumps(mapping["abi"], indent=2))

ids_map = {}
for key in mapping['methodIdentifiers']:
    func_id = mapping['methodIdentifiers'][key]
    ids_map[func_id] = [key]

with open(OUTPUT_IDS, 'w') as abi_file:
    abi_file.write(json.dumps(ids_map, indent=2))
