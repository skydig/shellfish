export GCC_PREFIX=
export BINDGEN_EXTRA_CLANG_ARGS_armv7_unknown_linux_gnueabihf="--sysroot=/opt/sysroot_arm-linux-gnueabihf"
source /opt/sysroot_arm-linux-gnueabihf/env-armhf.sh
cargo build --target=armv7-unknown-linux-gnueabihf --release -vv

