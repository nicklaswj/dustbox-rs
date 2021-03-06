test:
	cargo test -- --color always --nocapture

expensive-test:
	cargo test -- --color always --nocapture --ignored

bench:
	cargo bench

run:
	cargo run

run-release:
	cargo run --release

lint:
	# rm -rf target
	rustup run nightly cargo clippy

typos:
	speller . > spell

bindiff:
	vbindiff ~/dosbox-x/MEMDUMP.BIN emu_mem.bin
