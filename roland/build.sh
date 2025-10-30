#!/bin/bash

PI_USER=roland
PI_HOST=10.93.154.4
BUILD_TARGET=aarch64-unknown-linux-gnu

set -e

cargo build --target=${BUILD_TARGET}

scp target/${BUILD_TARGET}/debug/roland ${PI_USER}@${PI_HOST}:/home/${PI_USER}
