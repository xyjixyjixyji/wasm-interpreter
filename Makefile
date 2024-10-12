.PHONY: build build-tests run-tests clean

.DEFAULT_GOAL := all

all: build build-tests

build:
	cargo b --target x86_64-apple-darwin --release
	ln -sf target/x86_64-apple-darwin/release/wasm-interpreter-rs ./wasm-vm

build-debug:
	cargo b --target x86_64-apple-darwin
	ln -sf target/x86_64-apple-darwin/debug/wasm-interpreter-rs ./wasm-vm

build-tests:
	make -C tests

run-tests: build build-tests
	./grade.sh

clean:
	cargo clean
	rm -rf wasm-vm

