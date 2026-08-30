[private]
@default:
    just --list

lint:
    cargo clippy

[group('build')]
build-debug:
    cargo build

[group('build')]
build-release:
    cargo build --release

[group('test')]
test-debug:
    cargo test --workspace --locked

[group('test')]
test-release:
    cargo test --workspace --locked --release

[group('test')]
test-font-debug:
    cargo test --workspace --locked -- --ignored

[group('test')]
test-font-release:
    cargo test --workspace --locked --release -- --ignored

[group('Document')]
doc-lint:
    rumdl check

[group('Document')]
doc-fix:
    rumdl check --fix

tag:
    git tag $(cargo pkgid which-key-wayland | sed 's/.*#//')
