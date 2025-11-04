#! /usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

cleanup() {
    kill $TRUNK_PID 2>/dev/null || true
    exit
}

trap cleanup SIGINT SIGTERM EXIT

(cd gui && trunk watch) &
TRUNK_PID=$!

(cd web && SHAJARAH_DIST="dist" dx serve)
