#!/bin/sh

docker run --rm -v $PWD:/build_dir:Z -w /build_dir  -it liuchong/rustup:musl cargo build --release --target x86_64-unknown-linux-musl

