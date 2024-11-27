cargo install opencv
cargo build



cargo run --bin lab4_threaded shorter_soap.mp4
Lab 4 Video: https://vimeo.com/1023622996?share=copy


cargo run --bin lab5_simd shorter_soap.mp4
Lab 5 Video: https://vimeo.com/1028673815?share=copy



for crosscompilation:
install opencv dev libraries on target
copy target filesystem to use as sysroot (set of libraries for target)
using .cargo/config.toml, set sysroot to desired folder
compile with cargo build --target aarch64-unknown-linux-gnu
optionally, statically compile

