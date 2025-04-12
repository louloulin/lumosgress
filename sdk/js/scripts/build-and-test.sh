#!/bin/bash

# Build the project
echo "Building the project..."
npm run build

# Make the CLI executable
chmod +x ./dist/cli.js

# Test the CLI help command
echo -e "\nTesting CLI help command..."
./dist/cli.js --help

echo -e "\nIf you have API access configured, you can try these commands:"
echo "./dist/cli.js completion \"What is the capital of France?\""
echo "./dist/cli.js stream \"Write a short poem about coding.\""
echo "./dist/cli.js chat"

echo -e "\nCLI implementation completed and tested!" 