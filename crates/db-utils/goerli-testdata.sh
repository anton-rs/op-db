#!/bin/bash

# Download the Bedrock Goerli datadir from gcloud
echo "Downloading Bedrock Goerli datadir from gcloud..."
curl -L https://datadirs.optimism.io/goerli-bedrock.tar.zst --output goerli-bedrock.tar.zst
echo "Done. Untarring..."

# Untar the datadir
mkdir -p testdata/bedrock
tar --zstd -xvf goerli-bedrock.tar.zst -C testdata/bedrock
rm goerli-bedrock.tar.zst
echo "Done."

# echo "----------------------------------------------------------------------------------"
#
# # Download the Legacy Optimism Goerli datadir from gcloud
# echo "Downloading Legacy Optimism Goerli Archival datadir from gcloud..."
# curl -L https://datadirs.optimism.io/goerli-legacy-archival.tar.zst --output goerli-legacy-archival.tar.zst
# echo "Done Untarring..."
#
# # Untar the datadir
# mkdir -p testdata/legacy-archival
# tar --zstd -xvf goerli-legacy-archival.tar.zst -C testdata/legacy-archival
# rm goerli-legacy-archival.tar.zst
# echo "Done."
