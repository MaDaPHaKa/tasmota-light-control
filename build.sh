#!/usr/bin/env bash

set -euo pipefail

./tasmota-lights-control-fe/build.sh
./tasmota-lights-control-be/build.sh
