FROM ekidd/rust-musl-builder:stable

WORKDIR /home/rust/src/duck
RUN mkdir ../out
RUN cargo install cargo-watch 
EXPOSE 15825
ENTRYPOINT cargo watch -w Cargo.toml -w deny.toml -w src/ -x 'run --verbose --target-dir ../out  --features docker -- start --config ./data/duck.json'
