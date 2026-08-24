# Deployment Guide

## Overview

This guide covers how to deploy and integrate `code-intelligence` across environments: local development, CI/CD pipelines, containerized environments, and cloud infrastructure.

---

## Installation Methods

### 1. Automated Script Installation (Recommended)

Pre-built standalone binaries include all machine learning models compiled directly into the binary. No manual model setup or downloads are required.

#### **Linux & macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.sh | bash
```

#### **Windows (PowerShell as Administrator)**

```powershell
irm https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.ps1 | iex
```

#### **Windows (Command Prompt / Batch)**

Download and execute [`install.bat`](https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.bat) as Administrator.

---

### 2. Source Installation (Cargo)

**Prerequisites:**

- Rust 1.70+
- Cargo
- Git

```bash
# Clone the repository
git clone https://github.com/neontoshi/code-intelligence.git
cd code-intelligence

# Build and install the binary (embeds models/model.bin automatically)
cargo install --path .

# Verify installation
ci --version
```

---

### 3. Docker Deployment

```dockerfile
# Multi-stage Dockerfile
FROM rust:1.70-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --bin ci

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates git && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/ci /usr/local/bin/ci
ENTRYPOINT ["ci"]
```

```bash
# Build image
docker build -t code-intelligence .

# Run analysis directly on a mounted volume
docker run --rm -v $(pwd):/project code-intelligence analyze /project
```

---

## Environment Configuration

### Runtime Environment Variables

| Variable | Description | Default |
|----------|--------------|---------|
| `CI_MEMORY_LIMIT_MB` | Process memory ceiling, in MB | `4096` |
| `CI_CACHE_DIR` | Disk cache directory | `~/.cache/code-intelligence` |
| `CI_MODEL_PATH` | Path to a custom model override (optional) | *Uses embedded binary model* |
| `CI_LOG_LEVEL` | Logging level (`trace`, `debug`, `info`, `warn`, `error`) | `info` |
| `CI_THREADS` | Rayon thread-pool count | `CPU core count` |

### Optional Service Integrations

| Variable | Description |
|----------|--------------|
| `OLLAMA_HOST` | Local Ollama endpoint (e.g. `http://localhost:11434`) |
| `OPENAI_API_KEY` | OpenAI API key for LLM explanations |
| `ANTHROPIC_API_KEY` | Anthropic API key for Claude integration |
| `CI_NO_COLOR` | Suppress ANSI color escapes in output logs |
| `CI_JSON_LOGS` | Format all terminal logs as structured JSON |

### Configuration File (`config.toml`)

**Default path:** `~/.config/code-intelligence/config.toml`

```toml
[defaults]
# Note: 'model' is omitted by default to use the built-in embedded ML model
threshold = 0.92
verbose = false
llm_provider = "ollama"
llm_model = "phi:2.7b"

[projects]
"~/my-project" = { threshold = 0.92, project_type = "rust" }
```

---

## Model Architecture & Management

### Embedded Standalone Models

All release builds bundle default weights directly into memory via `include_bytes!`:

- **Dead Code Classifier:** `models/model.bin` (calibrated logistic regression on 46 features)
- **Duplicate Classifier:** `models/duplicate_model_v4.bin` (101 structural & token features)

### Custom Model Overrides (Optional)

If training custom weights for a domain-specific repository:

```bash
# 1. Train a custom classifier
ci train --data data/train.json --output custom_model.bin

# 2. Calibrate probabilities
ci calibrate --model custom_model.bin --data data/val.json --output custom_calibrated.bin

# 3. Use the custom model during analysis
ci analyze . --model custom_calibrated.bin
```

---

## CI/CD Pipeline Integrations

> **Note:** the GitHub Actions, GitLab CI, Jenkins, pre-commit, and Kubernetes examples below originally all ran `ci ci . --format json ...` — a doubled subcommand. Every other invocation in this doc (Cargo verification, the custom-model example, the Docker Compose `command:`, the pre-commit shell script) uses the single form `ci analyze`, so these five have been corrected to match. Flagging in case `ci ci` was actually intended as a distinct subcommand.

### GitHub Actions

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
        uses: actions/checkout@v4

      - name: Install Code Intelligence
        run: |
          curl -fsSL https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.sh | bash

      - name: Run Dead Code Analysis
        id: analysis
        run: |
          ci analyze . --format json --output dead_code_report.json --threshold 0.85

      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: dead-code-report
          path: dead_code_report.json

      - name: Check Failure Condition
        if: steps.analysis.outputs.dead_count != '0'
        run: |
          echo "Dead code detected exceeding threshold"
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
  image: debian:bookworm-slim
  before_script:
    - apt-get update && apt-get install -y curl ca-certificates git
    - curl -fsSL https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.sh | bash
  script:
    - ci analyze . --format json --output dead_code_report.json --threshold 0.85
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
                    curl -fsSL https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.sh | bash
                '''
            }
        }

        stage('Analyze') {
            steps {
                sh '''
                    ci analyze . --format json --output dead_code_report.json --threshold 0.85
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

---

## Pre-commit Hooks

### Git Pre-commit Hook

**File:** `.git/hooks/pre-commit`

```bash
#!/usr/bin/env bash

if git diff --cached --name-only | grep -q '\.'; then
    echo "🔍 Checking for dead code..."
    ci analyze . --cache

    if ci stats 2>/dev/null | grep -q "Pending: [1-9]"; then
        echo "❌ Commit rejected: Pending dead code found."
        echo "   Run 'ci list' to review findings"
        echo "   Run 'ci remove <name>' if deleted"
        echo "   Run 'ci keep <name> \"reason\"' to whitelist"
        exit 1
    fi
fi
```

### `.pre-commit-config.yaml`

```yaml
repos:
  - repo: local
    hooks:
      - id: dead-code
        name: Code Intelligence Dead Code Check
        entry: ci analyze
        language: system
        files: \.(rs|py|js|ts|tsx|go|java|dart|php|cs|cpp)$
        pass_filenames: false
        args: ['.', '--format=json', '--output=dead_code_report.json', '--threshold=0.85']
```

---

## Production & Container Orchestration

### Docker Compose

**File:** `docker-compose.yml`

```yaml
version: '3.8'

services:
  code-intelligence:
    build: .
    volumes:
      - .:/project
      - ci-cache:/cache
    environment:
      - CI_CACHE_DIR=/cache
      - CI_MEMORY_LIMIT_MB=4096
    entrypoint: ci
    command: analyze /project

volumes:
  ci-cache:
```

### Kubernetes Batch Job

**File:** `k8s-job.yaml`

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
        args: ["analyze", "/project", "--format", "json", "--output", "/results/report.json", "--threshold", "0.85"]
        volumeMounts:
        - name: project-data
          mountPath: /project
        - name: report-output
          mountPath: /results
        env:
        - name: CI_MEMORY_LIMIT_MB
          value: "4096"
      volumes:
      - name: project-data
        hostPath:
          path: /data/project
      - name: report-output
        hostPath:
          path: /data/results
      restartPolicy: Never
```

---

## Monitoring & Alerting

### Prometheus Metrics

```rust
use prometheus::{register_gauge, register_counter, Gauge, Counter};

lazy_static::lazy_static! {
    static ref DEAD_FUNCTIONS_COUNT: Gauge = register_gauge!(
        "ci_dead_functions_count",
        "Total unreferenced dead functions identified"
    ).unwrap();
    static ref ANALYSIS_DURATION_SECONDS: Gauge = register_gauge!(
        "ci_analysis_duration_seconds",
        "Pipeline execution wall-clock time in seconds"
    ).unwrap();
}
```

---

## Troubleshooting & FAQ

### Embedded Model Execution

**Q: Do I need to distribute `models/` with my production container?**

**A:** No. All models are embedded in the compiled binary. The container only needs the `ci` executable.

### Memory Limit Exceeded

**Q: How do I handle large monolith repositories (>50,000 functions)?**

**A:** Increase the allocation limit via environment variable and enable incremental disk caching:

```bash
export CI_MEMORY_LIMIT_MB=8192
ci analyze . --cache
```

### False Positive Whitelisting

**Q: What if a framework-dispatched function is marked as dead?**

**A:** Mark it as kept, to record the outcome in `.code-intelligence-outcomes.json`:

```bash
ci keep handleInternalWebhook "Called via dynamic reflection webhook dispatcher"
```
