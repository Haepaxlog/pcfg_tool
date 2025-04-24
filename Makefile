.PHONY: pcfg_tool
pcfg_tool:
	cargo build --release
	cp -p target/release/pcfg_tool .

.PHONY: clean
clean:
	rm -f pcfg_tool

.PHONY: deep-clean
deep-clean: clean
	cargo clean
