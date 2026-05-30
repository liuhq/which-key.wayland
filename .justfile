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
  cargo test

[group('test')]
test-release:
  cargo test --release

[group('Document')]
doc-lint:
  rumdl check

[group('Document')]
doc-fix:
  rumdl check --fix

tag:
  git tag $(cargo pkgid which-key-wayland | sed 's/.*#//')
