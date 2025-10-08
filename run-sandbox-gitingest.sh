#!/bin/bash

# Script to run gitingest in Docker container
# Usage: ./run-gitingest.sh <path-to-project>

set -e

# Check if path argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <path-to-project>"
    echo "Example: $0 ~/projects/myrepo"
    echo "Example: $0 https://github.com/user/repo"
    exit 1
fi

PROJECT_PATH="$1"

# Check if it's a URL or local path
if [[ "$PROJECT_PATH" == http* ]]; then
    # It's a URL - no need to check if directory exists
    echo "Processing GitHub repository: $PROJECT_PATH"
    OUTPUT_DIR="$(pwd)"
    
    # Run container with URL
    docker run --rm \
        -v "$OUTPUT_DIR:/output" \
        gitingest-runner "$PROJECT_PATH"
    
else
    # It's a local path
    # Convert to absolute path
    PROJECT_PATH=$(realpath "$PROJECT_PATH")
    
    # Check if the directory exists
    if [ ! -d "$PROJECT_PATH" ]; then
        echo "Error: Directory '$PROJECT_PATH' does not exist"
        exit 1
    fi
    
    echo "Processing local directory: $PROJECT_PATH"
    
    # Use the project directory as the output directory
    OUTPUT_DIR="$PROJECT_PATH"
    
    # Run container with local path mounted
    docker run --rm \
        -v "$PROJECT_PATH:/data" \
        -v "$OUTPUT_DIR:/output" \
        gitingest-runner /data
fi

echo "✅ gitingest completed successfully!"
echo "📄 Output saved to: $OUTPUT_DIR/digest.txt"
