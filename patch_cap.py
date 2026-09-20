import json

with open("src-tauri/capabilities/default.json", "r") as f:
    data = json.load(f)

data["permissions"] = [p for p in data["permissions"] if p not in ("fs:allow-read-file", "fs:allow-write-file", "fs:allow-exists")]

with open("src-tauri/capabilities/default.json", "w") as f:
    json.dump(data, f, indent=4)
