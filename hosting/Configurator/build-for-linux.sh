RUSTFLAGS="-C linker=x86_64-linux-musl-gcc -Ctarget-cpu=haswell -Ctarget-feature=+avx2" cargo build --release --target x86_64-unknown-linux-musl

version="999.999.999" ./package-server-images.sh
docker build /tmp/context/ -f $KEY -t registry.dev.cheetah.games/cheetah/platform/${VALUE}:${version}