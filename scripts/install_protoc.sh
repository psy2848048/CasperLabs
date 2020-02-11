#!/usr/bin/env bash

set -eux

# install protoc
PROTOBUF_VERSION=3.10.0
PROTOBUF_FILE="protoc-${PROTOBUF_VERSION}-${TRAVIS_OS_NAME}-x86_64.zip"
wget -v "https://github.com/google/protobuf/releases/download/v${PROTOBUF_VERSION}/${PROTOBUF_FILE}"
unzip "${PROTOBUF_FILE}" -d "${HOME}/protoc"
