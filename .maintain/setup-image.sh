#!/bin/bash
#
#
#

set -xe

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../


TAG_NAME=$(echo $GITHUB_REF | cut -d'/' -f 3)
GITHUB_SHA_SHORT=$(echo $GITHUB_SHA | cut -c1-7)

IMAGE_ORIGIN_NAME=darwinia:${TAG_NAME}

IMAGE_PREFIX=${REGISTRY_HOST:-'quay.io'}/darwinia-network
IMAGE_PUSH_NAME_TAG=${IMAGE_PREFIX}/${IMAGE_ORIGIN_NAME}
IMAGE_PUSH_NAME_SHA=${IMAGE_PREFIX}/darwinia:sha-${GITHUB_SHA_SHORT}

DOCKERFILE_NAME=Dockerfile.darwinia.x86_64-linux-gnu

chmod +x ${WORK_PATH}/deploy/darwinia

docker build \
  -t ${IMAGE_ORIGIN_NAME} \
  -f ${WORK_PATH}/.maintain/docker/${DOCKERFILE_NAME} \
  ${WORK_PATH} || exit 1

docker tag ${IMAGE_ORIGIN_NAME} ${IMAGE_PUSH_NAME_TAG}
docker tag ${IMAGE_ORIGIN_NAME} ${IMAGE_PUSH_NAME_SHA}

docker push ${IMAGE_PUSH_NAME_TAG}
docker push ${IMAGE_PUSH_NAME_SHA}
