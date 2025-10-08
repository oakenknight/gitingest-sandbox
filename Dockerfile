FROM python:3.12-slim

# Install gitingest
RUN pip install --no-cache-dir gitingest

# Set working directory to /data where files will be mounted
WORKDIR /data

# Run gitingest on a provided path or URL
ENTRYPOINT ["/bin/bash", "-c", "gitingest \"$1\" -o /output/digest.txt"]
