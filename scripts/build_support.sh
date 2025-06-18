#!/bin/bash

# DocumentDB Linux Installation Script
# This script installs all dependencies required to build DocumentDB on Linux
# Based on the .devcontainer/Dockerfile
# Supports Ubuntu, Debian, CentOS, RHEL, Fedora, and other Linux distributions

# fail if trying to reference a variable that is not set.
set -u
# exit immediately if a command exits with a non-zero status
set -e


# Configuration
PG_VERSION=${PG_VERSION:-16}
CITUS_VERSION=${CITUS_VERSION:-12}
POSTGRES_INSTALL_ARG=${POSTGRES_INSTALL_ARG:-""}

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


export INSTALL_DEPENDENCIES_ROOT=/tmp/install_setup
mkdir -p $INSTALL_DEPENDENCIES_ROOT

# Copy setup_versions.sh which decides versions of the dependencies to install.
cp $scriptDir/setup_versions.sh $INSTALL_DEPENDENCIES_ROOT/



# Function to install libbson
install_libbson() {
    echo "Installing libbson..."
    # Check if libbson is already installed
    if pkg-config --exists libbson-1.0; then
        echo "libbson is already installed. Skipping installation."
    else
        cp -p "$scriptDir/install_libbson.sh" /tmp/install_setup/
        export MAKE_PROGRAM=cmake
        /tmp/install_setup/install_libbson.sh
        echo "libbson installed"
    fi
}

install_intelmathlib() {
    echo "Installing Intel Math Library..."
    # Check if Intel Math Library is already installed
    if pkg-config --exists intelmathlib; then
        echo "Intel Math Library is already installed. Skipping installation."
    else
        cp -p "$scriptDir/install_intelmathlib_local.sh" /tmp/install_setup/
        export MAKE_PROGRAM=cmake
        /tmp/install_setup/install_intelmathlib_local.sh
        echo "Intel Math Library installed"
    fi
}


main() {
    echo "Starting DocumentDB installation on Linux..."
    echo "PostgreSQL Version: ${PG_VERSION}"
    echo "Citus Version: ${CITUS_VERSION}"
    
    # Install libbson
    install_libbson
    install_intelmathlib
    
}


# Run main function
main "$@" 