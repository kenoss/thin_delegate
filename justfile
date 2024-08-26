check:
  cargo build && cargo clippy && cargo fmt -- --check

check-strict:
  export CARGO_TARGET_DIR=target/check-strict RUSTFLAGS='-D warnings'; just check

check-warn:
  export CARGO_TARGET_DIR=target/check-strict RUSTFLAGS='-D warnings'; clear; cargo build --color always |& head -n 32

run *ARGS:
  cargo run {{ARGS}}

test *ARGS:
  cargo test --features unstable_delegate_to {{ARGS}}

nextest-run *ARGS:
  cargo nextest run --features unstable_delegate_to {{ARGS}}
