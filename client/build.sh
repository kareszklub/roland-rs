#!/bin/bash

export PUBLIC_ROLAND_IP=$(avahi-resolve -4n roland.local | awk '{print $2}')

vite dev
