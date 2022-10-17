build:
	cargo build --release

fmt:
	cargo fmt --all

lint:
	cargo clippy --all

clean:
	cargo clean

run-dsp *args:
	cargo run --release -- {{args}} dsp

run-noise *args:
	cargo run --release -- {{args}} noise

run-bytebeats formula *args:
	cargo run --release -- {{args}} bytebeats "{{formula}}"
