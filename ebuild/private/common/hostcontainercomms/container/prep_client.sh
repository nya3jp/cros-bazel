#!/bin/bash

set -e
set -x

HOST_UID="$1"
HOST_GID="$2"
SERVER_ADDRESS="$3"

echo -n "${HOST_UID}" > /helpers/host_uid
echo -n "${HOST_GID}" > /helpers/host_gid
echo -n "${SERVER_ADDRESS}" > /helpers/server_address
