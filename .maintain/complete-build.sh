#!/bin/bash
#
#
#

set -xe

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../

cd ${WORK_PATH}/deploy/bin

echo -e '\e[1;32m🔑 Generating File(s) Hash\e[0m'

md5sum * > ../md5sums.txt
sha256sum * > ../sha256sums.txt

mv ../md5sums.txt .
mv ../sha256sums.txt .

ls
