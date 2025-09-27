#! /usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

(trap 'kill 0' SIGINT; \
 bash -c 'cd gui; trunk watch' & \
 bash -c 'cd web; SHAJARAH_DIST="dist" dx serve')
