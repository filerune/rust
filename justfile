set shell := ["bash", "-cu"]
set windows-shell := ["pwsh", "-Command"]

fusion := "packages/fusion"

# Default action
_:
    just lint
    just fmt
    just test

# Lint code
lint:
    ls-lint -config ./.ls-lint.yaml
    typos
    cargo check
    cargo clippy
    cargo test -p filerune -- --nocapture
    cargo test -p filerune_fusion --features all -- --nocapture

# Format code
fmt:
    cargo fmt

# Run test
test:
    cargo test -p tests -- --nocapture

# Run benchmark
bench:
    cargo bench -p bench

# Publish fusion package as dry-run
publish-try-fusion:
    cd ./{{fusion}} && cargo publish --dry-run

# Publish all packages as dry-run
publish-try:
    just publish-try-fusion

# Publish fusion package
publish-fusion:
    cd ./{{fusion}} && cargo publish

# Publish all packages
publish:
    just publish-fusion

# Clean up
clean:
    rm -rf ./bench/.media
    rm -rf ./tests/.media

clean-all:
    just clean
    
    cargo clean
