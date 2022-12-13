build:
	# ... build ...
	cargo +nightly build
	cargo +nightly build --features cbordata
	cargo +stable build
	cargo +stable build --features cbordata
	#
	# ... test ...
	cargo +nightly test --no-run
	cargo +nightly test --no-run --features cbordata
	cargo +stable test --no-run
	cargo +stable test --no-run --features cbordata
	#
	# ... bench ...
	cargo +nightly bench --no-run
	cargo +nightly bench --no-run --features cbordata
	#
	# ... doc ...
	cargo +nightly doc
	cargo +nightly doc --features cbordata
	cargo +stable doc
	cargo +stable doc --features cbordata
	#
	# ... meta commands ...
	cargo +nightly clippy --all-targets --all-features

test:
	# ... test ...
	cargo +nightly test
	cargo +nightly test --features cbordata
	cargo +stable test --no-run
	cargo +stable test --no-run --features cbordata

lint:
	cargo fmt
	cargo clippy --all-targets -- -D warnings

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps

bench:
	# ... test ...
	cargo +nightly bench
	cargo +nightly bench --features cbordata
	cargo +stable test --no-run

flamegraph:
	echo "not an executable"

prepare: build test bench
	check.sh check.out
	perf.sh perf.out

clean:
	cargo clean
	rm -f check.out perf.out flamegraph.svg perf.data perf.data.old
