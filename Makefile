.PHONY: build build-tests run-tests clean

.DEFAULT_GOAL := all

all: build build-tests

build:
	cargo build
	ln -sf target/debug/wasm-interpreter-rs ./wasm-vm

build-tests:
	make -C tests

run-tests: build build-tests
	./grade.sh

clean:
	cargo clean
	rm -rf wasm-vm

