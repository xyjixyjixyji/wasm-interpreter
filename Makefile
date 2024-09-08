.PHONY: build build-tests run-tests clean

build:
	cargo check
	cargo b
	cp target/debug/wasm-interpreter-rs ./wasm-vm

build-tests:
	make -C tests

run-tests: build build-tests
	./grade.sh

clean:
	cargo clean
	make -C tests clean

