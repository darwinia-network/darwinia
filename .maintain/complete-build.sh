#!/bin/sh
#
#
#

set -x

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../

### test start
#mkdir -p bin
#cp Cargo.toml bin/
#cp README.adoc bin/
### test end


cd ${WORK_PATH}/bin


echo -e '\e[1;32m🔑 Generating File(s) Hash\e[0m'
for f in $(ls); do
    md5sum $f >> ../md5sums.txt
done
for f in $(ls); do
    sha256sum $f >> ../sha256sums.txt
done
mv ../md5sums.txt .
mv ../sha256sums.txt .

pwd
ls
