check:
  cargo build && cargo clippy --tests --examples && cargo fmt -- --check

check-strict:
  export CARGO_TARGET_DIR=target/check-strict RUSTFLAGS='-D warnings'; just check

check-warn:
  export CARGO_TARGET_DIR=target/check-strict RUSTFLAGS='-D warnings'; clear; cargo build --color always |& head -n 32

run *ARGS:
  cargo run {{ARGS}}

test *ARGS:
  cargo test --features test_smithay {{ARGS}}

test-ci *ARGS:
  cargo test {{ARGS}}

nextest-run *ARGS:
  cargo nextest run --features test_smithay {{ARGS}}
