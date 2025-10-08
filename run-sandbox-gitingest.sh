#!/bin/bash

# SECURE VERSION - Script to run gitingest in Docker container with security restrictions
# Usage: ./run-sandbox-gitingest.sh <path-to-project>

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
    # It's a URL - only allow network access for GitHub
    echo "Processing GitHub repository: $PROJECT_PATH"
    OUTPUT_DIR="$(pwd)"
    
    # Run container with URL - restrict network to only allow GitHub
    docker run --rm \
        --network none \
        --add-host=github.com:140.82.112.3 \
        --add-host=api.github.com:140.82.112.6 \
        -v "$OUTPUT_DIR:/output" \
        gitingest-runner "$PROJECT_PATH"
    
else
    # It's a local path
    PROJECT_PATH=$(realpath "$PROJECT_PATH")
    
    if [ ! -d "$PROJECT_PATH" ]; then
        echo "Error: Directory '$PROJECT_PATH' does not exist"
        exit 1
    fi
    
    echo "Processing local directory: $PROJECT_PATH"
    OUTPUT_DIR="$PROJECT_PATH"
    
    # Run container with NO NETWORK ACCESS for local files
    docker run --rm \
        --network none \
        --read-only \
        --tmpfs /tmp:rw,noexec,nosuid,size=100m \
        -v "$PROJECT_PATH:/data:ro" \
        -v "$OUTPUT_DIR:/output:rw" \
        gitingest-runner /data
fi

echo "✅ gitingest completed successfully!"
echo "📄 Output saved to: $OUTPUT_DIR/digest.txt"

# Security cleanup - stop any running containers and prune data
echo "🧹 Performing security cleanup..."
echo "   Stopping any running gitingest containers..."
docker ps --filter "ancestor=gitingest-runner" --format "table {{.ID}}\t{{.Image}}\t{{.Status}}" | grep -v "CONTAINER ID" | while read container_id rest; do
    if [ ! -z "$container_id" ]; then
        echo "   Stopping container: $container_id"
        docker stop "$container_id" >/dev/null 2>&1 || true
        docker rm "$container_id" >/dev/null 2>&1 || true
    fi
done

echo "   Removing any orphaned gitingest containers..."
docker ps -a --filter "ancestor=gitingest-runner" --format "{{.ID}}" | while read container_id; do
    if [ ! -z "$container_id" ]; then
        echo "   Removing container: $container_id"
        docker rm "$container_id" >/dev/null 2>&1 || true
    fi
done

echo "   Pruning unused Docker data..."
docker system prune -f >/dev/null 2>&1 || true

echo "🔒 Security cleanup completed!"
echo "   All gitingest containers have been stopped and removed"
echo "   Unused Docker data has been pruned"
