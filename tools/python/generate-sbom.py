import argparse
import hashlib
import os
import requests
import toml
import uuid
import json
import subprocess
from urllib.parse import urlparse

def get_current_commit(cargo_lock_dir):
    return subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=cargo_lock_dir).decode('utf-8').strip()

def get_current_repo_url(cargo_lock_dir):
    return subprocess.check_output(['git', 'config', '--get', 'remote.origin.url'], cwd=cargo_lock_dir).decode('utf-8').strip()

def parse_args():
    parser = argparse.ArgumentParser(description="Generate software bill of materials (SBOM) for Rust project")
    
    parser.add_argument(
        '--cargo-lock',
        dest='cargo_lock_path',
        default='./Cargo.lock',
        help="Path to Cargo.lock file. Default - ./Cargo.lock"
    )
    
    parser.add_argument(
        '--cargo-toml',
        dest='cargo_toml_path',
        default='./Cargo.toml',
        help="Path to Cargo.toml file. Default - ./Cargo.toml"
    )
    
    parser.add_argument(
        '--initial-sbom',
        dest='initial_sbom_path',
        default='initial-sbom.json',
        help="Optional. Path to initial SBOM JSON file if need to append existing SBOM. Default - initial-sbom.json. Will ignore if file doesn't exist."
    )
    
    parser.add_argument(
        '--sbom-output',
        dest='sbom_output_path',
        default='sbom.json',
        help="Path to output SBOM JSON file. Default - sbom.json"
    )
    
    parser.add_argument(
        '--project-src',
        dest='project_src_path',
        default='./',
        help="Path to the Rust project source if not in root git directory. Not relates to local file system path. Relates to path inside repo structure. For example we can use v5_x/v5.1.0/git-remote-gosh which means https://github.com/gosh-sh/gosh/v5_x/v5.1.0/git-remote-gosh"
    )
    
    parser.add_argument(
        '--project-commit',
        dest='project_commit',
        default=None,
        help="Commit of the project. Default - commit parsed with 'git rev-parse HEAD' command in dir where Cargo.lock is located."
    )
    
    parser.add_argument(
        '--project-url',
        dest='project_url',
        default=None,
        help="URL of the project's repository. Default - project URL parsed with 'git config --get remote.origin.url' command in dir where Cargo.lock is located."
    )
    
    args = parser.parse_args()

    # Calculate the directory containing the Cargo.lock file
    cargo_lock_dir = os.path.dirname(os.path.abspath(args.cargo_lock_path))

    # Parse default values based on Cargo.lock directory
    if args.project_commit is None:
        args.project_commit = get_current_commit(cargo_lock_dir)

    if args.project_url is None:
        args.project_url = get_current_repo_url(cargo_lock_dir)

    return args

if __name__ == '__main__':
    parsed_args = parse_args()

    # Use the current directory as the base for relative paths
    current_directory = os.getcwd()

    # Check if the provided paths are relative or absolute
    if not os.path.isabs(parsed_args.cargo_lock_path):
        # If relative, join with the current directory
        CARGO_LOCK_PATH = os.path.join(current_directory, parsed_args.cargo_lock_path)
    else:
        CARGO_LOCK_PATH = parsed_args.cargo_lock_path

    if not os.path.isabs(parsed_args.cargo_toml_path):
        # If relative, join with the current directory
        CARGO_TOML_PATH = os.path.join(current_directory, parsed_args.cargo_toml_path)
    else:
        CARGO_TOML_PATH = parsed_args.cargo_toml_path

    # Check and handle INITIAL_SBOM_PATH
    if not os.path.isabs(parsed_args.initial_sbom_path):
        INITIAL_SBOM_PATH = os.path.join(current_directory, parsed_args.initial_sbom_path)
    else:
        INITIAL_SBOM_PATH = parsed_args.initial_sbom_path

    # Check and handle SBOM_OUTPUT_PATH
    if not os.path.isabs(parsed_args.sbom_output_path):
        SBOM_OUTPUT_PATH = os.path.join(current_directory, parsed_args.sbom_output_path)
    else:
        SBOM_OUTPUT_PATH = parsed_args.sbom_output_path
    
    PROJECT_SRC_PATH = parsed_args.project_src_path
    PROJECT_URL = parsed_args.project_url
    PROJECT_COMMIT = parsed_args.project_commit
    TMP_FILE_PATH = os.path.abspath('tmp_file')

    print("Config:")
    print(f"=================================================================")
    print(f"CARGO_LOCK_PATH: {CARGO_LOCK_PATH}")
    print(f"CARGO_TOML_PATH: {CARGO_TOML_PATH}")
    print(f"INITIAL_SBOM_PATH: {INITIAL_SBOM_PATH}")
    print(f"SBOM_OUTPUT_PATH: {SBOM_OUTPUT_PATH}")
    print(f"PROJECT_URL: {PROJECT_URL}")
    print(f"PROJECT_COMMIT: {PROJECT_COMMIT}")
    print(f"PROJECT_SRC_PATH: {PROJECT_SRC_PATH}")
    print(f"=================================================================")


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
                    "vendor": "GOSH",
                    "name": "anytree",
                    "version": "1.0.0"
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
        },
        {
          "name": "base_image",
          "value": "teamgosh/gosh-rust:1.71"
        },
        {
          "name": "prerun",
          "value": "make copy_abi"
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

# Process dependencies from Cargo.lock
for package in cargo_lock.get('package', []):
    name = package.get('name')
    version = package.get('version')
    
    if 'source' not in package:
        print(f"Warning: Skipping package {name} due to lack of source")
        continue

    source = package.get('source')
    tmp_file = os.path.abspath('tmp_file')

    try:
        if "git" in source and '#' in source:
            mime_type = "cargo/git"
            url, commit = source.split('#')
            url = url.split('?')[0].replace("git+", "")
            clone_and_archive(url, commit, tmp_file)
            hashes = get_hashes(tmp_file)
            external_references = [{"url": url, "type": "distribution"}]
            #properties = [{"name": "commit", "value": commit}]
            if 'git+' in source and '?tag=' in source:
                tag = source.split('?tag=')[1].split('#')[0]  # Extract tag from the URL
                properties = [{"name": "commit", "value": commit}, {"name": "tag", "value": tag}]
            else:
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
        bom["components"].append(component)
        
        # Add additional component if name and GitHub repo name mismatch
        if external_references and urlparse(external_references[0]["url"]).netloc == 'github.com':
            repo_name = os.path.splitext(os.path.basename(urlparse(external_references[0]["url"]).path))[0]
            if repo_name != name:
                mismatch_component = {
                    "bom-ref": f"{repo_name}_{version.replace('.', '_')}_{uuid.uuid4()}",
                    "type": "library",
                    "name": repo_name,
                    "version": version,
                    "mime-type": mime_type,
                    "externalReferences": external_references,
                    "hashes": [{"alg": alg, "content": content} for alg, content in hashes.items()],
                }
                if properties:
                    mismatch_component["properties"] = properties
                bom["components"].append(mismatch_component)
        
    finally:
        if os.path.isfile(tmp_file):
            os.remove(tmp_file)



# Process patch section from Cargo.lock
for package in cargo_lock.get('patch', {}).get('unused', []):
    name = package.get('name')
    version = package.get('version')
    
    if 'source' not in package:
        print(f"Warning: Skipping package {name} due to lack of source")
        continue

    source = package.get('source')
    tmp_file = os.path.abspath('tmp_file')

    try:
        if "git" in source and '#' in source:
            mime_type = "cargo/git"
            url, commit = source.split('#')
            url = url.split('?')[0].replace("git+", "")
            clone_and_archive(url, commit, tmp_file)
            hashes = get_hashes(tmp_file)
            external_references = [{"url": url, "type": "distribution"}]
            if 'git+' in source and '?tag=' in source:
                tag = source.split('?tag=')[1].split('#')[0]  # Extract tag from the URL
                properties = [{"name": "commit", "value": commit}, {"name": "tag", "value": tag}]
            else:
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
        bom["components"].append(component)
        
    finally:
        if os.path.isfile(tmp_file):
            os.remove(tmp_file)

# Remove the existing component, if any, with the same name and version
components = bom.get("components", [])
bom["components"] = [component for component in components if component.get("name") != project_name and component.get("version") != project_version]

# Add the new component at the beginning of the components list
bom["components"].insert(0, main_component)

# Write SBOM back to the same file
with open(SBOM_OUTPUT_PATH, 'w') as f:
    json.dump(bom, f, indent=2)
    print(f"Updated SBOM written to {SBOM_OUTPUT_PATH}")
