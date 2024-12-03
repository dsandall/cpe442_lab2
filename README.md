## lab 4 and 5 instructions 
cargo run --bin lab4_threaded shorter_soap.mp4
Lab 4 Video: https://vimeo.com/1023622996?share=copy

cargo run --bin lab5_simd shorter_soap.mp4
Lab 5 Video: https://vimeo.com/1028673815?share=copy


## for Lab 6 : RPI cluster network
make sure to set the local IP of your host node, then compile (right now the host must be rpi/aarch64, but there's no reason this must be the case for you)  

#### run send.sh
just send those binaries to the rpi targets and run 1 host and X clients
targets must install opencv library (i'd like to compile this statically... coming soon?)

---
# Build from source


## for building and running on RPI
cargo install opencv
cargo build



## for crosscompilation:
because compiling this code on a raspberry pi is a terrible experience, and because crosscompilation is not super simple when using c libraries in your rust program

### set up your target
install opencv dev libraries on target

### copy sysroot from the target to the compilation machine
copy entire target filesystem to compilation machine using scp or rsync 
sysroot is used as set of shared libraries that are assumed to be on the target  

### tell the compiler where to find said target libraries
using .cargo/config.toml, set sysroot to desired folder
compile with cargo build --target aarch64-unknown-linux-gnu
#### .cargo/config.toml already has build rules for the aarch64-unknown-llinux-gnu (rpi3 and up processor core)




