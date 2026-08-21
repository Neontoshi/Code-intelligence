## Document 7: `docs/deployment.md`

```markdown
# Deployment Guide

## Overview

This guide covers how to deploy `code-intelligence` in various environments: local development, CI/CD pipelines, and production systems.

---

## Installation Methods

### 1. Source Installation

**Prerequisites:**
- Rust 1.70+
- Cargo
- Git

```bash
# Clone the repository
git clone https://github.com/neontoshi/Code-intelligence
cd code-intelligence

# Build and install all binaries
cargo install --path .

# Or build specific binary
cargo build --release --bin ci

# Verify installation
ci --version
```

### 2. Pre-built Binary

```bash
# Download the latest release
curl -L https://github.com/neontoshi/Code-intelligence/releases/latest/download/ci -o ci
chmod +x ci
sudo mv ci /usr/local/bin/

# Verify
ci --version
```

### 3. Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.70-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --bin ci

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/ci /usr/local/bin/ci
ENTRYPOINT ["ci"]
```

```bash
# Build image
docker build -t code-intelligence .

# Run analysis
docker run -v $(pwd):/project code-intelligence analyze /project
```

---

## Environment Configuration

### Required Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CI_MEMORY_LIMIT_MB` | Memory limit in MB | 4096 |
| `CI_CACHE_DIR` | Cache directory | `~/.cache/code-intelligence` |
| `CI_MODEL_PATH` | Default model path | `models/dead_code_model_v2.bin` |
| `CI_LOG_LEVEL` | Log level (debug, info, warn, error) | `info` |
| `CI_THREADS` | Number of threads | CPU cores |

### Optional Environment Variables

| Variable | Description |
|----------|-------------|
| `OLLAMA_HOST` | Ollama server URL |
| `OPENAI_API_KEY` | OpenAI API key |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `CI_NO_COLOR` | Disable colored output |
| `CI_JSON_LOGS` | JSON formatted logs |

### Configuration File

**Location:** `~/.config/code-intelligence/config.toml`

```toml
[defaults]
model = "models/dead_code_model_v2.bin"
threshold = 0.85
verbose = false
llm_provider = "ollama"
llm_model = "phi:2.7b"

[projects]
"~/my-project" = {
    threshold = 0.92,
    project_type = "rust"
}
```

---

## Model Management

### Downloading Models

```bash
# Download latest model
curl -L https://github.com/neontoshi/Code-intelligence/releases/latest/download/model.bin -o models/dead_code_model_v2.bin

# Or train your own
ci train --data data/train.json --output my_model.bin
```

### Model Directory Structure

```
models/
├── dead_code_model_v2.bin      # Main model
├── dead_code_model_v2.json     # Model metadata
├── duplicate_model_v2.bin      # Duplicate detection model
└── README.md                   # Model documentation
```

### Model Versioning

```bash
# List available models
ls -la models/*.bin

# Use specific model
ci config set model models/dead_code_model_v2.bin

# Train new version
ci train --data data/train_v3.json --output models/dead_code_model_v3.bin
```

---

## CI/CD Integration

### GitHub Actions

**Full Example:**

```yaml
name: Dead Code Analysis

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  dead-code-check:
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v3
        
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
          
      - name: Install code-intelligence
        run: |
          git clone https://github.com/neontoshi/Code-intelligence.git /tmp/ci
          cd /tmp/ci
          cargo install --path .
          
      - name: Download model
        run: |
          mkdir -p models
          curl -L https://github.com/neontoshi/Code-intelligence/releases/latest/download/model.bin -o models/dead_code_model_v2.bin
          
      - name: Configure CI
        run: |
          ci config set model models/dead_code_model_v2.bin
          ci config set threshold 0.85
          
      - name: Run dead code analysis
        id: analysis
        run: |
          ci ci . --format json --output dead_code_report.json --threshold 0.85
          
      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: dead-code-report
          path: dead_code_report.json
          
      - name: Fail if dead code found
        if: steps.analysis.outputs.dead_count != '0'
        run: |
          echo "❌ Found dead code!"
          cat dead_code_report.json | jq .
          exit 1
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - analyze

dead-code:
  stage: analyze
  image: rust:latest
  before_script:
    - apt-get update && apt-get install -y git
    - git clone https://github.com/neontoshi/Code-intelligence.git /tmp/ci
    - cd /tmp/ci && cargo install --path .
    - ci config set model /tmp/ci/models/dead_code_model_v2.bin
  script:
    - ci ci . --format json --output dead_code_report.json
  artifacts:
    paths:
      - dead_code_report.json
  only:
    - merge_requests
    - main
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any
    
    stages {
        stage('Install') {
            steps {
                sh '''
                    git clone https://github.com/neontoshi/Code-intelligence.git /tmp/ci
                    cd /tmp/ci
                    cargo install --path .
                    ci config set model /tmp/ci/models/dead_code_model_v2.bin
                '''
            }
        }
        
        stage('Analyze') {
            steps {
                sh '''
                    ci ci . --format json --output dead_code_report.json
                '''
            }
        }
        
        stage('Publish Report') {
            steps {
                archiveArtifacts artifacts: 'dead_code_report.json'
            }
        }
    }
}
```

### CircleCI

```yaml
# .circleci/config.yml
version: 2.1

jobs:
  dead-code:
    docker:
      - image: rust:latest
    steps:
      - checkout
      - run:
          name: Install code-intelligence
          command: |
            git clone https://github.com/neontoshi/Code-intelligence.git /tmp/ci
            cd /tmp/ci
            cargo install --path .
            ci config set model /tmp/ci/models/dead_code_model_v2.bin
      - run:
          name: Run analysis
          command: ci ci . --format json --output dead_code_report.json
      - store_artifacts:
          path: dead_code_report.json

workflows:
  version: 2
  build:
    jobs:
      - dead-code
```

---

## Pre-commit Hooks

### Git Pre-commit Hook

**File: `.git/hooks/pre-commit`**

```bash
#!/usr/bin/env bash

# Exit if no staged files
if git diff --cached --name-only | grep -q '\.'; then
    echo "🔍 Checking for dead code..."
    
    # Run analysis on staged files
    ci analyze . --cache
    
    # Check if any dead code found
    if ci stats 2>/dev/null | grep -q "Pending: [1-9]"; then
        echo "❌ Commit rejected: Dead code found!"
        echo "   Run 'ci list' to see findings"
        echo "   Run 'ci remove <name>' if deleted"
        echo "   Run 'ci keep <name> \"reason\"' to whitelist"
        exit 1
    fi
fi
```

### Pre-commit Framework

**File: `.pre-commit-config.yaml`**

```yaml
repos:
  - repo: local
    hooks:
      - id: dead-code
        name: Dead code check
        entry: ci ci
        language: system
        files: \.(rs|py|js|ts|go|java)$
        pass_filenames: false
        args: ['.', '--format=json', '--output=dead_code_report.json']
```

---

## Docker Deployment

### Running with Docker

```bash
# Build the image
docker build -t code-intelligence:latest .

# Run analysis on current directory
docker run -v $(pwd):/project code-intelligence:latest analyze /project

# With custom model
docker run -v $(pwd):/project -v $(pwd)/models:/models code-intelligence:latest analyze /project --model /models/model.bin

# With Ollama
docker run -v $(pwd):/project --network host code-intelligence:latest analyze /project --llm
```

### Docker Compose

**File: `docker-compose.yml`**

```yaml
version: '3.8'

services:
  code-intelligence:
    build: .
    volumes:
      - .:/project
      - ./models:/models
      - ./cache:/cache
    environment:
      - CI_MODEL_PATH=/models/dead_code_model_v2.bin
      - CI_CACHE_DIR=/cache
      - CI_MEMORY_LIMIT_MB=4096
    entrypoint: ci
    command: analyze /project
```

---

## Production Deployment

### Kubernetes

**File: `k8s-job.yaml`**

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: code-intelligence-analysis
spec:
  template:
    spec:
      containers:
      - name: ci
        image: code-intelligence:latest
        command: ["ci"]
        args: ["ci", "/project", "--format", "json", "--output", "/results/report.json"]
        volumeMounts:
        - name: project
          mountPath: /project
        - name: results
          mountPath: /results
        - name: models
          mountPath: /models
        env:
        - name: CI_MODEL_PATH
          value: /models/dead_code_model_v2.bin
        - name: CI_MEMORY_LIMIT_MB
          value: "4096"
      volumes:
      - name: project
        hostPath:
          path: /data/project
      - name: results
        hostPath:
          path: /data/results
      - name: models
        hostPath:
          path: /data/models
      restartPolicy: Never
```

### AWS Lambda

**File: `lambda.rs`**

```rust
use code_intelligence::Pipeline;
use aws_lambda_events::event::cloudwatch_events::CloudWatchEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

async fn handler(_event: LambdaEvent<CloudWatchEvent>) -> Result<(), Error> {
    // Analyze project from S3
    let project_path = "/tmp/project";
    
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(project_path).await?;
    
    // Upload results to S3
    let report = analysis.to_json();
    // ... upload to S3
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
```

---

## Monitoring & Alerting

### Prometheus Metrics

```rust
// Expose metrics via Prometheus
use prometheus::{register_gauge, register_counter};

lazy_static! {
    static ref DEAD_COUNT: Gauge = register_gauge!("ci_dead_functions", "Dead functions found").unwrap();
    static ref ANALYSIS_DURATION: Gauge = register_gauge!("ci_analysis_duration_seconds", "Analysis duration").unwrap();
}
```

### Alerting Rules

```yaml
# prometheus-rules.yaml
groups:
  - name: code-intelligence
    rules:
      - alert: HighDeadCodeCount
        expr: ci_dead_functions > 100
        for: 1h
        annotations:
          summary: "High dead code count"
          description: "Found {{ $value }} dead functions"
      
      - alert: AnalysisFailed
        expr: ci_analysis_success == 0
        for: 5m
        annotations:
          summary: "Analysis failed"
          description: "Dead code analysis failed"
```

---

## Backup & Recovery

### Backing Up Models

```bash
# Backup models
tar -czf models-backup-$(date +%Y%m%d).tar.gz models/

# Restore models
tar -xzf models-backup-20260101.tar.gz
```

### Backing Up Training Data

```bash
# Backup training data
tar -czf training-data-$(date +%Y%m%d).tar.gz data/

# Restore training data
tar -xzf training-data-20260101.tar.gz
```

---

## Troubleshooting Deployment

### Common Issues

#### Model Not Found

```bash
# Error: Model file not found

# Solution: Download model
ci config set model models/dead_code_model_v2.bin

# Or use absolute path
ci config set model /usr/local/share/code-intelligence/models/model.bin
```

#### Permission Denied

```bash
# Error: Permission denied

# Solution: Fix permissions
chmod +x /usr/local/bin/ci

# Or install to user directory
cargo install --path . --root ~/.local
```

#### Memory Issues

```bash
# Error: Memory limit exceeded

# Solution: Increase memory limit
export CI_MEMORY_LIMIT_MB=8192

# Or reduce scope
ci analyze . --max-files 1000
```

#### Cache Issues

```bash
# Error: Cache corruption

# Solution: Clear cache
rm -rf .code-intelligence-cache

# Or disable cache
ci analyze . --no-cache
```

---

## Performance Tuning

### Thread Configuration

```bash
# Use all cores
export RAYON_NUM_THREADS=$(nproc)

# Limit threads
export RAYON_NUM_THREADS=4

# Disable parallel processing
export RAYON_NUM_THREADS=1
```

### Memory Configuration

```bash
# Set memory limit
export CI_MEMORY_LIMIT_MB=4096

# Disable expensive features
ci analyze . --no-cycle-detection --no-feature-cache
```

### Cache Configuration

```bash
# Use SSD for cache
export CI_CACHE_DIR=/fast-ssd/cache

# Pre-warm cache
ci analyze . --cache --warm-up

# Clear stale cache
ci cache clear
```

---

## Security Hardening

### File System Access

```bash
# Run with limited permissions
sudo -u nobody ci analyze /project

# Use read-only mount
docker run -v $(pwd):/project:ro code-intelligence analyze /project
```

### Network Access

```bash
# Disable network for offline analysis
ci analyze . --no-llm --offline

# Or restrict network
docker run --network none code-intelligence analyze /project
```

### API Keys

```bash
# Use environment variables
export OPENAI_API_KEY=sk-...

# Use secure storage
ci config set openai-api-key $(vault read -field=key secret/openai)
```

---

## Update Strategy

### Version Updates

```bash
# Check current version
ci --version

# Update from source
cd code-intelligence
git pull
cargo install --path .

# Update via package manager
cargo install --force --path .
```

### Model Updates

```bash
# Download latest model
curl -L https://github.com/neontoshi/Code-intelligence/releases/latest/download/model.bin -o models/model.bin

# Validate model
ci validate-model models/model.bin

# Update config
ci config set model models/model.bin
```

### Rollback

```bash
# Rollback version
git checkout v0.1.0
cargo install --path .

# Rollback model
ci config set model models/model_v1.bin
```

---

## Summary Checklist

### Pre-Deployment

- [ ] Install Rust and Cargo
- [ ] Clone repository
- [ ] Download model
- [ ] Configure environment
- [ ] Test analysis on sample project

### Production Deployment

- [ ] Set up CI/CD pipeline
- [ ] Configure monitoring
- [ ] Set up alerting
- [ ] Document configuration
- [ ] Create backup strategy

### Post-Deployment

- [ ] Run first analysis
- [ ] Verify results
- [ ] Set up scheduled runs
- [ ] Configure notifications
- [ ] Document any customizations
```

---
