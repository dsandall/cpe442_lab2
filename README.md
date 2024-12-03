
---
# Lab specific instructions

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

# Hello!
## how to use this code
- pull the repo
- install dependencies
- if on rpi,
-   just build/run it using ``cargo run``
- else: 
-   goto Crosscompilation


# Installing dependencies
#### how to setup rust and opencv for debian (similar for other flavors of *nix)
## Follow these steps on the machine you wish to write code on (build environment)
- run the rust install script, or install the rustup package (google: install rust)
- install build-essential package
- follow "opencv" crate setup info for ubuntu/debian: ``apt install libopencv-dev clang libclang-dev``
-   or try cargo install opencv

## for building and running on RPI

- cargo build and run!



## Crosscompilation:
because compiling this code on a raspberry pi is a terribly slow experience, and because crosscompilation is not super simple when using c libraries in your rust program

- if crosscompiling: 
-   install deps on client (target architecture)
-   get the sysroot
-   compile binaries
-   send binaries to clients
  
### set up your target
install opencv dev libraries on target (as mentioned earlier)

### copy sysroot from the target to the compilation machine
copy entire target filesystem to compilation machine using scp or rsync 
sysroot is used as set of shared libraries that are assumed to be on the target  

### tell the compiler where to find said target libraries
using .cargo/config.toml, set sysroot to desired folder
add support for new target architecture ``cargo add --target aarch64-unknown-linux-gnu``
compile with ``cargo build --target aarch64-unknown-linux-gnu`` (this is the default as of lab 5, where arm intrinsics limit the platform of the code anyway)
#### .cargo/config.toml already has build rules for the aarch64-unknown-linux-gnu (rpi3 and up processor core)


