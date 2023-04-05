#!/usr/bin/env bash

set -e

ARGS=()
# If LEADER_HOST is not set, then we are the leader
if [ -z "$LEADER_HOST" ]; then
    echo "Starting leader"
    # RING=""
else
    echo "Starting follower"
    LEADER_IP=$(getent hosts $LEADER_HOST | awk '{ print $1 }')
    ARGS+=("--ring" "$LEADER_IP:42000")
fi

MY_IP=$(hostname -i)

ARGS+=("--listen" "$MY_IP:42000")

echo "Starting with args: ${ARGS[@]}"

server ${ARGS[@]}
