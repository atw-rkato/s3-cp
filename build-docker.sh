#!/bin/bash
set -eu

START_TIME=$(date +%s)

APP_NAME=s3-cp
BASE_IMAGE_TAG=1.59.0
BASE_IMAGE="joseluisq/rust-linux-darwin-builder:${BASE_IMAGE_TAG}"

UUID=$(uuidgen | tr "[:upper:]" "[:lower:]")
echo "${UUID}"

APP_IMAGE="${APP_NAME}:${UUID}"

EXIST_BASE_IMAGE=false
if [ -n "$(docker image ls -q "${BASE_IMAGE}")" ]; then
  EXIST_BASE_IMAGE=true
else
  docker pull "${BASE_IMAGE}"
fi

docker build -t "${APP_IMAGE}" . \
  --build-arg APP_NAME="${APP_NAME}" \
  --build-arg BASE_IMAGE_TAG="${BASE_IMAGE_TAG}" \
  --progress plain
docker container run --rm -v "$(pwd)/dist:/dist" "${APP_IMAGE}"

docker image rm "${APP_IMAGE}"
if ! "$EXIST_BASE_IMAGE"; then
  docker image rm "${BASE_IMAGE}"
fi

END_TIME=$(date +%s)

ELAPSED_SEC=$((END_TIME - START_TIME))

docker builder prune -f --filter "until=${ELAPSED_SEC}s"
