#!/bin/bash
# ===============================================================================
#
#         FILE: build_images.sh
#
#        USAGE: build_images.sh [version] [branch]
#
#  DESCRIPTION:
#
#      OPTIONS:  version: default git hash
#                branch: default master
# REQUIREMENTS:  ---
#         BUGS:  ---
#        NOTES:  ---
#       AUTHOR:  YOUR NAME (),
#      COMPANY:
#      VERSION:  1.0
#      CREATED:
#     REVISION:  ---
# ===============================================================================

# version=${1:-"none"}
# branch=${2:-"master"}
bin_git=$(which git)
bin_docker=$(which docker)
src=$(pwd)"/../"
version='none'
branch='master'
startup_cmd='-dev'
docker_volume=''
docker_ports=''
container_name='darwinia'
start_docker=0

usage() {
    echo "Usage: $0 [-v version] [-b branch] [-s startup docker] [-h]"
    echo "version: default git version"
    echo "branch: default master"
    echo "start docker: default start -dev"
    exit 1
}

while getopts 'v:b:s:h' flag
do
    case "${flag}" in
      v) 
        if [ $(echo $OPTARG | grep -E '^[0-9a-zA-Z_\-\s\.]+$' -c) -eq 1 ]
        then 
            version=${OPTARG}
        fi
        ;;
      b) 
        if [ $(echo $OPTARG | grep -E '^[0-9a-zA-Z_\-\s\.]+$' -c) -eq 1 ]
        then 
            branch=${OPTARG}
        fi
        ;;
      s)
        start_docker=1
        ;;
      ?) usage;;
    esac
done


usage() {
    echo "$0 [-v version] [-b branch] [-h]"
    exit 1
}

check_branch() {
    [ $(${bin_git} branch | grep -E "^(\*)?\s+${branch}$" -c) -eq 0 ] && echo "Branch ${branch} not find" && exit 1
}

check_env() {
    [ "${bin_git}x" == "x" ] && echo "git not find" && exit 1
    [ "${bin_docker}x" == "x" ] && echo "docker not find" && exit 1
}

main() {
    check_env
    check_branch
    cd $src
    current_branch=$(git symbolic-ref --short -q HEAD 2>/dev/null)
    if [ $current_branch != $branch ]
    then
        $bin_git check $branch
    fi
    if [ $version == "none" ]
    then
        version=$(git rev-parse HEAD 2>/dev/null)
    fi
    # 开始build
    echo "Starting build ${branch}-${version}"
    $bin_docker build . -t darwinia:${branch}-${version} 
    # build 成功判断
    rt=$?
    if [ $rt -ne 0 ]
    then
        echo "$bin_docker build . -t ${branch}-${version} failed, return $rt"
        exit $rt
    fi
    [ $start_docker -eq 0 ] && exit 0

    # 成功，检查镜像是否启动过，启动过就干掉
    # $bin_docker ps -a --filter "name=${container_name}" --format "{{.Names}}-{{.Image}}"
    # 启动镜像，传递进参数
    container_status=$($bin_docker ps -a --filter "name=${container_name}" --format "{{.Names}}-{{.Image}}")
    if [ "${container_status}x" != "x" ]:
    then
        c_n = $(echo $container_status | cut -d "-" -f 1)
        c_i = $(echo $container_status | cut -d "-" -f 2)
        echo "Stop and remove ${container_name}"
        $bin_docker stop ${container_name}
        rt=$?
        if [ $rt -ne 0 ]
        then
            echo "$bin_docker stop ${container_name} failed, return $rt"
            exit $rt
        fi
        $bin_docker rm ${container_name}
        rt=$?
        if [ $rt -ne 0 ]
        then
            echo "$bin_docker rm ${container_name} failed, return $rt"
            exit $rt
        fi        
    fi
    $bin_docker run -d --name=${container_name} -P darwinia:${branch}-${version} $startup_cmd
    rt=$?
    if [ $rt -ne 0 ]
    then
        echo "$bin_docker rm ${container_name} failed, return $rt"
        exit $rt
    fi
    echo "Start ${container_name} done, enjoy."
    exit 0 
}

main

