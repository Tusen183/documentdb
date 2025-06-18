#!/bin/bash

# fail if trying to reference a variable that is not set.
set -u
# exit immediately if a command exits with a non-zero status
set -e
source="${BASH_SOURCE[0]}"
while [[ -h $source ]]; do
   scriptroot="$( cd -P "$( dirname "$source" )" && pwd )"
   source="$(readlink "$source")"

   # if $source was a relative symlink, we need to resolve it relative to the path where the
   # symlink file was located
   [[ $source != /* ]] && source="$scriptroot/$source"
done
scriptDir="$( cd -P "$( dirname "$source" )" && pwd )"
echo "scriptDir: $scriptDir"

. $scriptDir/setup_versions.sh
MONGO_DRIVER_VERSION=$(GetLibbsonVersion)
export INSTALL_DEPENDENCIES_ROOT=/tmp/install_setup
mkdir -p /tmp/install_setup

if [ "${INSTALLDESTDIR:-""}" == "" ]; then
    INSTALLDESTDIR="/usr";
fi

if [ "${MAKE_PROGRAM:-""}" == "" ]; then
    MAKE_PROGRAM="cmake"
fi

echo "Starting libbson installation..."
echo "Version: $MONGO_DRIVER_VERSION"
echo "Installation directory: $INSTALLDESTDIR"
echo "Build tool: $MAKE_PROGRAM"

pushd $INSTALL_DEPENDENCIES_ROOT
echo "Checking for existing mongo-c-driver source..."

if [ -f "$scriptDir/dependencies/mongo-c-driver-$MONGO_DRIVER_VERSION.tar.gz" ]; then
    echo "Found existing package in dependencies directory, copying..."
    cp "$scriptDir/dependencies/mongo-c-driver-$MONGO_DRIVER_VERSION.tar.gz" ./
else
    echo "Downloading mongo-c-driver source..."
    curl -L https://github.com/mongodb/mongo-c-driver/releases/download/$MONGO_DRIVER_VERSION/mongo-c-driver-$MONGO_DRIVER_VERSION.tar.gz -o ./mongo-c-driver-$MONGO_DRIVER_VERSION.tar.gz
fi

echo "Extracting source files..."
tar -xzvf ./mongo-c-driver-$MONGO_DRIVER_VERSION.tar.gz -C $INSTALL_DEPENDENCIES_ROOT --transform="s|mongo-c-driver-$MONGO_DRIVER_VERSION|mongo-c-driver|"

# remove the tar file
echo "Cleaning up downloaded archive..."
rm -rf ./mongo-c-driver-$MONGO_DRIVER_VERSION.tar.gz

echo "Configuring build..."
cd $INSTALL_DEPENDENCIES_ROOT/mongo-c-driver/build
$MAKE_PROGRAM -DENABLE_MONGOC=ON -DMONGOC_ENABLE_ICU=OFF -DENABLE_ICU=OFF -DCMAKE_C_FLAGS="-fPIC -g" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=$INSTALLDESTDIR ..

echo "Building and installing..."
make clean && make -sj$(cat /proc/cpuinfo | grep -c "processor") install
popd

if [ "${CLEANUP_SETUP:-"0"}" == "1" ]; then
    echo "Cleaning up build files..."
    rm -rf $INSTALL_DEPENDENCIES_ROOT/mongo-c-driver
fi

echo "libbson installation completed successfully!"
