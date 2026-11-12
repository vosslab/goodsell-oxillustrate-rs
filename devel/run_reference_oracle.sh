#!/usr/bin/env bash
# Development-only Fortran oracle; no installed/native application dependency.
set -e
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"
source source_me.sh
python3 devel/reference_oracle.py
