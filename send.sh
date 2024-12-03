#!zsh

# Define IP addresses
IP_0="10.0.1.154"
IP_1="10.0.1.17" 
IP_2="10.0.1.156"

IP_3="10.0.1.20"
IP_4="10.0.1.23"
IP_7="10.0.1.21"

IP_ICEPI="10.0.1.152"

#debug or release
PROFILE="dev"

# Build the binaries
cargo build --bin lab6_host --profile $PROFILE 
cargo build --bin lab6_compute --profile $PROFILE
cargo build --bin lab5_simd --profile $PROFILE

# SCP commands
scp ./target/aarch64-unknown-linux-gnu/debug/{lab6_host,lab6_compute,lab5_simd} admin@$IP_ICEPI:/home/admin/
scp ./target/aarch64-unknown-linux-gnu/debug/{lab6_host,lab6_compute,lab5_simd} thebu@$IP_0:/home/thebu/
scp ./target/aarch64-unknown-linux-gnu/debug/{lab6_host,lab6_compute,lab5_simd} thebu@$IP_1:/home/thebu/
scp ./target/aarch64-unknown-linux-gnu/debug/{lab6_host,lab6_compute,lab5_simd} thebu@$IP_2:/home/thebu/
scp ./target/aarch64-unknown-linux-gnu/debug/{lab6_host,lab6_compute,lab5_simd} thebu@$IP_3:/home/thebu/
scp ./target/aarch64-unknown-linux-gnu/debug/{lab6_host,lab6_compute,lab5_simd} thebu@$IP_4:/home/thebu/
scp ./target/aarch64-unknown-linux-gnu/debug/{lab6_host,lab6_compute,lab5_simd} thebu@$IP_7:/home/thebu/

# # debug or release
# PROFILE="release"

# # Build the binaries
# cargo build --bin lab6_host --profile $PROFILE 
# cargo build --bin lab6_compute --profile $PROFILE
# cargo build --bin lab5_simd --profile $PROFILE

# # SCP commands
# scp ./target/aarch64-unknown-linux-gnu/release/{lab6_host,lab6_compute,lab5_simd} admin@$IP_ICEPI:/home/admin/
# scp ./target/aarch64-unknown-linux-gnu/release/{lab6_host,lab6_compute,lab5_simd} thebu@$IP_1:/home/thebu/
# scp ./target/aarch64-unknown-linux-gnu/release/{lab6_host,lab6_compute,lab5_simd} thebu@$IP_2:/home/thebu/
# scp ./target/aarch64-unknown-linux-gnu/release/{lab6_host,lab6_compute,lab5_simd} thebu@$IP_0:/home/thebu/