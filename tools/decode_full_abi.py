import json

INPUT_FILE = 'contracts/l1/out/Elock.sol/Elock.json'
OUTPUT_ABI = 'resources/elock.abi.json'
OUTPUT_IDS = 'resources/identifiers.json'
OUTPUT_EVENTS = 'resources/events.json'

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

events = []
for value in mapping['abi']:
    if value["type"] == "event":
        params = []
        for input in value["inputs"]:
            params.append({"name": input["name"], "type": input["type"], "indexed": input["indexed"]})
        event = {"name": value["name"], "params": params, "anonymous": value["anonymous"]}
        events.append(event)

event_map = {}
for node in mapping["ast"]["nodes"]:
    if node["nodeType"] == "ContractDefinition":
        for contract_node in node["nodes"]:
            if contract_node["nodeType"] == "EventDefinition":
                for event in events:
                    if event["name"] == contract_node["name"]:
                        event_map[f'0x{contract_node["eventSelector"]}'] = event

with open(OUTPUT_EVENTS, 'w') as abi_file:
    abi_file.write(json.dumps(event_map, indent=2))
