#!/bin/sh
#
#
#

set -x

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../


BRANCH_NAME=$(echo $GITHUB_REF | cut -d'/' -f 3)
GITHUB_SHA_SHORT=$(echo $GITHUB_SHA | cut -c1-8)
TAG_NAME=${GITHUB_REF#refs/*/}

IMAGE_PREFIX=quay.io/darwinia-network
IMAGE_ORIGIN_NAME=darwinia:${TAG_NAME}
IMAGE_PUSH_NAME=${IMAGE_PREFIX}/${IMAGE_ORIGIN_NAME}

DOCKERFILE_NAME=Dockerfile.darwinia.x86_64-linux-gnu

docker build \
  -t ${IMAGE_PUSH_NAME} \
  -f ${WORK_PATH}/.maintain/docker/${DOCKERFILE_NAME} \
  ${WORK_PATH} || exit 1

#docker tag ${IMAGE_ORIGIN_NAME} ${IMAGE_PUSH_NAME}

docker push ${IMAGE_PUSH_NAME}
