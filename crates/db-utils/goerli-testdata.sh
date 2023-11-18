#!/bin/bash

echo "Downloading Bedrock Goerli datadir from gcloud..."

# Download the Bedrock Goerli datadir from gcloud
curl -L https://datadirs.optimism.io/goerli-bedrock.tar.zst --output goerli-bedrock.tar.zst

echo "Done. Untarring..."

# Untar the datadir
tar --zstd -xvf goerli-bedrock.tar.zst -C testdata
rm goerli-bedrock.tar.zst

echo "Done."
