#!/bin/bash
set -e

project_dir=$(pwd)
client_dir_from_protodir=../client

if [[ ($OSTYPE == 'darwin'*)  ]]; then
run_protoc() {
  protofile=$1
  protodir=$2
  grpc_out_path=$3
  mkdir -p $grpc_out_path
  scripts/bin/mac/protoc \
    --proto_path=$protodir \
    --plugin=protoc-gen-grpc=scripts/bin/mac/grpc_csharp_plugin \
    --grpc_out=$grpc_out_path \
    --csharp_out=$grpc_out_path \
    --experimental_allow_proto3_optional \
    $protofile
}
fi

if [[ ($OSTYPE == 'linux'*)  ]]; then
run_protoc() {
  protofile=$1
  protodir=$2
  grpc_out_path=$3
  mkdir -p $grpc_out_path
  scripts/bin/lin/protoc \
    --proto_path=$protodir \
    --plugin=protoc-gen-grpc=scripts/bin/lin/grpc_csharp_plugin \
    --grpc_out=$grpc_out_path \
    --csharp_out=$grpc_out_path \
    --experimental_allow_proto3_optional \
    $protofile
}
fi

if [[ ($OSTYPE == 'msys'*)  ]]; then
run_protoc() {
  protofile=$1
  protodir=$2
  grpc_out_path=$3
  mkdir -p $grpc_out_path
  scripts/bin/win/protoc.exe \
    --proto_path=$protodir \
    --plugin=protoc-gen-grpc=scripts/bin/win/grpc_csharp_plugin.exe \
    --grpc_out=$grpc_out_path \
    --csharp_out=$grpc_out_path \
    --experimental_allow_proto3_optional \
    $protofile
}
fi
