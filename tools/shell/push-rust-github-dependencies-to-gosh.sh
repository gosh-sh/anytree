#!/bin/bash

set -ex

PROJECT_DIR="/opt/sam-anytree/gosh/v5_x/v5.1.0/git-remote-gosh"
WALLET_KEYS="/opt/sam-anytree/keys/gosh.keys.json"

# PROFILE ADDR
PUB_ADDR="0:7610d6ea8cf362caa8c80dd0258acb07a00cc21986b80f0c7110dc5bdcb05b6c"

# COPIED FROM DAO UI
DAO_ADDR="0:5a2cd8bcfb8200f85d8454f5f9fe4c947d05f526c69a2968210a4d2883809b20"
# COPIED FROM CLONE REPOSITORY IN UI
DEST_ADDR="0:b00a7a5a24740e4a7d6487d31969732f1febcaea412df5cc307400818055ad58"

SOURCE="https://github.com/tonlabs/"
ORG="tonlabs"

# Set the destination repository URL
destination_base_url="gosh://$DEST_ADDR/git-remote-deps"

# Retrieve the list of dependencies from the Cargo.lock file with the specified source
dependencies=$(awk -F '"' '/^name =/ { name=$2 } /^version =/ { version=$2 } /^source =/ { source=$2 } name && version && index(source, "'$SOURCE'") { print name"@"version; name=""; version=""; source="" }' Cargo.lock.2 | sort -u)

# Create a temporary directory for the cloned repositories
tmp_dir=$(mktemp -d)

# Progress file
progress_file="progress.txt.2"

# Check if the progress file exists
if [ -f "$progress_file" ]; then
    # Progress file exists, read all successfully pushed dependencies
    readarray -t success_list < "$progress_file"
else
    # Progress file doesn't exist, start from the beginning
    success_list=()
fi

# Push all branches of the dependencies to the corresponding destination repository
for dep in $dependencies; do
    # Extract the package name and version
    package_name=$(echo "$dep" | cut -d'@' -f1)
    package_version=$(echo "$dep" | cut -d'@' -f2)
    package="${package_name}-${package_version//./-}"

    # Skip already pushed dependencies
    if [[ " ${success_list[@]} " =~ " ${package} " ]]; then
        continue
    fi

    echo ==================== START PUSHING $package ===================================

    ### run <DAO ADDR>
    wallet_addr=$(tonos-cli -u network.gosh.sh -j run $DAO_ADDR getAddrWallet \
        "{\"pubaddr\":\"$PUB_ADDR\",\"index\":0}" \
        --abi "$PROJECT_DIR/../contracts/gosh/goshdao.abi.json" |
        sed -n '/value0/ p' | cut -d'"' -f 4)

    tonos-cli -u network.gosh.sh call --abi "$PROJECT_DIR/../contracts/gosh/goshwallet.abi.json" --sign "$WALLET_KEYS" "$wallet_addr" \
        AloneDeployRepository \
        "{\"nameRepo\":\"$package\",\"descr\":\"\",\"previous\":null}"

    # Create the destination repository URL using the same name
    destination_repo="${destination_base_url}/${package}"

    cd "$tmp_dir"

    retries=0
    while [ $retries -lt 3 ]; do
        if git clone "$destination_repo"; then
            break
        fi
        retries=$((retries+1))
        sleep 60s
    done

    if [ $retries -eq 3 ]; then
        echo "Failed to push $package after 3 retries."
        exit 1
    fi

    cd $PROJECT_DIR
    git_source=$(grep -A2 'name = "'$package_name'"$' Cargo.lock.2 | grep -oP 'source = "\K[^"]+' | sed 's/^git+//')
    git_url=$(grep -A2 'name = "'$package_name'"$' Cargo.lock.2 | grep -oP 'source = "\K[^"]+' | sed 's/^git+//' | awk -F '[?#]' '{print $1}')

    repo_name=$(basename "$git_url" | sed 's/\.git.*$//')
    commit_id=$(echo "$git_source" | sed 's/.*#//')
    cd "$tmp_dir"
    git clone "$git_url"
    cd $tmp_dir/$repo_name
    git checkout $commit_id
    rm -rf .git
    cd ..
    cp -r $repo_name/* $package/
    cd $package
    git add .
    git commit -m 'push dependency' || true

    # Retry the git push command for a maximum of 3 times
    retries=0
    while [ $retries -lt 3 ]; do
        if git push $destination_repo; then
            break
        fi
        retries=$((retries+1))
        sleep 60s
    done

    if [ $retries -eq 3 ]; then
        echo "Failed to push $package after 3 retries."
        exit 1
    fi

    rm -rf "$tmp_dir/$package_name-$package_version"
    rm -rf "$tmp_dir/$repo_name"

    # Add the successfully pushed dependency to the list
    success_list+=("$package")

    # Update the progress file with the list of successfully pushed dependencies
    printf "%s\n" "${success_list[@]}" > "$PROJECT_DIR/$progress_file"

    echo ==================== END PUSHING $package ===================================
done

# Cleanup
rm -rf "$tmp_dir"
