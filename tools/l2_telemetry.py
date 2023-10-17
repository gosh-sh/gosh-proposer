import subprocess
import json
from supabase import create_client, Client
import os

from dotenv import load_dotenv

load_dotenv()

SUPABASE_URL = os.environ.get('SUPABASE_URL')
SUPABASE_API_KEY = os.environ.get('SUPABASE_API_KEY')


def execute_cmd(command: str, work_dir="."):
    global WAS_ERROR
    command = f"cd {work_dir} && {command}"
    try:
        output = subprocess.check_output(command, shell=True).decode("utf-8")
    except subprocess.CalledProcessError as e:
        output = e.output.decode("utf-8")
        print(f"Command `{command}` execution failed: {output}")
        exit(1)

    return output.strip()


def write_to_supabase():
    supabase: Client = create_client(SUPABASE_URL, SUPABASE_API_KEY)
    for i in range(1):
        telemetry = execute_cmd("l2-telemetry")
        mapping = json.loads(telemetry)
        print(mapping)
        data, count = supabase.table('l2_state') \
            .insert(mapping) \
            .execute()
        print(data, count)


write_to_supabase()
