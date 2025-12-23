#!/bin/bash

PI_USER=roland
PI_HOST=$(avahi-resolve -4n roland.local 2>/dev/null | awk '{print $2}')
BUILD_TARGET=aarch64-unknown-linux-gnu

set -e

cargo build --target=${BUILD_TARGET}

scp target/${BUILD_TARGET}/debug/roland ${PI_USER}@${PI_HOST}:/home/${PI_USER}
