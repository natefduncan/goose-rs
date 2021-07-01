import json

try:
    with open("output.json", "r") as f:
        data = json.load(f)
    print(f"JSON is valid and has {len(data)} records.")
except:
    print("JSON is not valid.")