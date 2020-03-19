release:
	cargo clean;
	cargo +stable build;
	cargo +nightly build;
	cargo +stable doc;
	cargo +nightly clippy --all-targets --all-features;
	cargo +nightly test;
	cargo +stable test;
	cargo +nightly bench;
