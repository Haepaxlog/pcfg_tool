pcfg_tool:
	cargo build --release
	cp -p target/release/pcfg_tool .

.PHONY: clean
clean:
	cargo clean
