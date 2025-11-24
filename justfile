
runreal:
  cargo run --

check:
  cargo fmt
  cargo test
  cargo check
  cargo clippy -- -D warnings
  @echo "ALL PASSED"

