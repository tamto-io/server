#!/usr/bin/env bash

# Usage:
# ./run-nodes.sh -n [num_nodes] -p [start_port] -l [leader]
#
# num_nodes: number of nodes to start (default: 3)
# start_port: port to start on (default: 42000)
# -l: leader host, if not set then the leader node will be started

set -e

NUM_NODES=3
START_PORT=42000
LISTEN_IP="[::1]"

while getopts "n:p:l:" opt; do
    case $opt in
        n)
            NUM_NODES=$OPTARG
            ;;
        p)
            START_PORT=$OPTARG
            ;;
        l)
            LEADER=$OPTARG
            ;;
        \?)
            echo "Invalid option: -$OPTARG" >&2
            exit 1
            ;;
    esac
done

function cleanup {
    echo "Waiting for jobs to finish"
    wait
}

trap 'trap " " SIGTERM; kill 0; wait; cleanup' SIGINT SIGTERM

echo "Starting $NUM_NODES nodes"
echo "Start port: $START_PORT"

if [ -z "$LEADER" ]; then
    LEADER="$LISTEN_IP:$START_PORT"
    ARGS+=("--listen" "$LEADER")
    START_PORT=$((START_PORT + 1))
    NUM_NODES=$((NUM_NODES - 1))

    echo "Starting leader with args: ${ARGS[@]}"

    nohup ./target/release/server ${ARGS[@]} &
fi

for i in $(seq 1 $NUM_NODES); do
    ARGS=("--listen" "$LISTEN_IP:$START_PORT" "--ring" "$LEADER")
    echo "Starting follower with args: ${ARGS[@]}"

    nohup ./target/release/server ${ARGS[@]} &

    START_PORT=$((START_PORT + 1))
done

wait
