import hashlib
import os
import requests
import toml
import uuid
import json
import subprocess
import concurrent.futures
from urllib.parse import urlparse
import tempfile

CARGO_LOCK_PATH = 'Cargo.lock'
CARGO_TOML_PATH = 'Cargo.toml'
INITIAL_SBOM_PATH = 'initial-sbom.json' # if need to append 
SBOM_OUTPUT_PATH = 'sbom.json'
PROJECT_URL = 'https://github.com/gosh-sh/gosh.git'
PROJECT_COMMIT = '08d9325d8df759ca833a60a66fcc6b2b8c060a87'
PROJECT_SRC_PATH = 'v5_x/v5.1.0/git-remote-gosh'
PROJECT_VENDOR = 'GOSH'

# Load Cargo.lock
with open(CARGO_LOCK_PATH) as f:
    cargo_lock = toml.load(f)

# Load Cargo.toml
with open(CARGO_TOML_PATH) as f:
    cargo_toml = toml.load(f)

# Get project details from Cargo.toml
project_name = cargo_toml.get('package', {}).get('name')
project_version = cargo_toml.get('package', {}).get('version')
bin_targets = cargo_toml.get('bin', [])
project_bin = bin_targets[0].get('name') if bin_targets else None

# Initialize BOM dictionary
if os.path.exists(INITIAL_SBOM_PATH):
    # Load existing BOM
    with open(INITIAL_SBOM_PATH) as f:
        bom = json.load(f)
else:
    # Predefined template for the initial BOM
    bom = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.4",
        "version": 1,
        "metadata": {
            "tools": [
                {
                    "vendor": PROJECT_VENDOR,
                    "name": project_name,
                    "version": project_version
                }
            ],
            "component": {
                "type": "application",
                "name": project_bin,
                "properties": [
                    {
                        "name": "platform",
                        "value": "linux",
                    }
                ],
            },
        },
        "components": [],
    }

# Main project component data
main_component = {
    "bom-ref": f"{project_name}_{project_version.replace('.', '_')}_{uuid.uuid4()}",
    "type": "application",
    "name": f"{project_name}",
    "version": f"{project_version}",
    "externalReferences": [
        {
            "url": PROJECT_URL,
            "type": "distribution"
        }
    ],
    "properties": [
        {
            "name": "commit",
            "value": PROJECT_COMMIT
        },
        {
            "name": "target",
            "value": "cargo/project"
        },
        {
            "name": "src_path",
            "value": PROJECT_SRC_PATH
        }
    ]
}

def get_hashes(file_path):
    with open(file_path,"rb") as f:
        bytes = f.read() # Read file bytes
        return {
            "MD5": hashlib.md5(bytes).hexdigest(),
            "SHA-1": hashlib.sha1(bytes).hexdigest(),
            "SHA-256": hashlib.sha256(bytes).hexdigest(),
            "SHA-512": hashlib.sha512(bytes).hexdigest(),
        }

def download_file(url, target_path):
    print(f"Downloading file from {url}")
    r = requests.get(url, stream=True)
    if r.status_code == 200:
        with open(target_path, 'wb') as f:
            for chunk in r.iter_content(1024):
                f.write(chunk)
        print(f"Downloaded file to {target_path}")

def clone_and_archive(url, commit, target_path):
    print(f"Cloning repository from {url}")
    subprocess.run(['git', 'clone', url, 'repo'], check=True)
    os.chdir('repo')
    subprocess.run(['git', 'checkout', commit], check=True)
    subprocess.run(['git', 'archive', '-o', target_path, 'HEAD'], check=True)
    os.chdir('../')
    subprocess.run(['rm', '-rf', 'repo'], check=True)
    print(f"Cloned repository and created archive at {target_path}")

def download_component(component, tmp_file):
    name = component.get('name')
    version = component.get('version')
    source = component.get('source')

    try:
        if "git" in source and '#' in source:
            mime_type = "cargo/git"
            url, commit = source.split('#')
            url = url.split('?')[0].replace("git+", "")
            clone_and_archive(url, commit, tmp_file)
            hashes = get_hashes(tmp_file)
            external_references = [{"url": url, "type": "distribution"}]
            properties = [{"name": "commit", "value": commit}]
        else:
            mime_type = "cargo/registry"
            url = f"https://crates.io/api/v1/crates/{name}/{version}/download"
            download_file(url, tmp_file)
            hashes = get_hashes(tmp_file)
            external_references = [{"url": url, "type": "distribution"}]
            properties = []

        component = {
            "bom-ref": f"{name}_{version.replace('.', '_')}_{uuid.uuid4()}",
            "type": "library",
            "name": name,
            "version": version,
            "mime-type": mime_type,
            "externalReferences": external_references,
            "hashes": [{"alg": alg, "content": content} for alg, content in hashes.items()],
        }
        if properties:
            component["properties"] = properties
        return component
    finally:
        if os.path.isfile(tmp_file):
            os.remove(tmp_file)

# Process dependencies from Cargo.lock using parallelism
bom_components = []
with concurrent.futures.ThreadPoolExecutor() as executor:
    future_to_component = {}
    with tempfile.TemporaryDirectory() as tmp_dir:
        for package in cargo_lock.get('package', []):
            tmp_file = os.path.join(tmp_dir, f"{package.get('name')}_{package.get('version')}.tmp")
            future_to_component[executor.submit(download_component, package, tmp_file)] = package

        for future in concurrent.futures.as_completed(future_to_component):
            component = future_to_component[future]
            try:
                result = future.result()
                if result:
                    bom_components.append(result)
            except Exception as e:
                print(f"Error processing package {component.get('name')}: {e}")

# Remove the existing component, if any, with the same name and version
bom["components"] = [component for component in bom.get("components", []) if component.get("name") != project_name and component.get("version") != project_version]

# Add the new component at the beginning of the components list
bom["components"] = [main_component] + bom_components + bom["components"]

# Write SBOM back to the same file
with open(SBOM_OUTPUT_PATH, 'w') as f:
    json.dump(bom, f, indent=2)
    print(f"Updated SBOM written to {SBOM_OUTPUT_PATH}")
