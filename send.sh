#!zsh

cargo build --bin lab6_host
cargo build --bin lab6_compute

scp ./target/aarch64-unknown-linux-gnu/debug/{lab6_host,lab6_compute} thebu@10.0.1.156:/home/thebu/
scp ./target/aarch64-unknown-linux-gnu/debug/{lab6_host,lab6_compute} admin@10.0.1.152:/home/admin/