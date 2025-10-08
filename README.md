# Gitingesters

A secure Rust CLI wrapper for [gitingest](https://github.com/coderamp-labs/gitingest) with Docker isolation and maximum security features.

## Installation

```bash
cargo install gitingesters
```

Or from source:
```bash
git clone https://github.com/oakenknight/gitingest-sandbox.git
cd gitingest-sandbox
cargo install --path .
```

## Usage

### Build Docker image
```bash
gitingesters build
```

### Process local directory
```bash
gitingesters run /path/to/project /path/to/output
```

### Process GitHub repository
```bash
gitingesters run-url https://github.com/user/repo /path/to/output
```

## Security Features

- 🔒 **Network isolation** - No internet access for local files
- 📁 **Read-only mounts** - Source code mounted as read-only
- 🗑️ **Auto cleanup** - Removes containers after execution
- ✅ **GitHub-only URLs** - Restricted network access for repositories

## Output

Creates `digest.txt` in the output directory with LLM-friendly text content of your repository.
