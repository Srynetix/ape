all:
	just --list

build:
	cargo build --release

fmt:
	cargo fmt --all

lint:
	cargo clippy --all

clean:
	cargo clean

run-dsp *args:
	cargo run --release --bin ape-cli -- {{args}} dsp

run-noise *args:
	cargo run --release --bin ape-cli -- {{args}} noise

run-bytebeats formula *args:
	cargo run --release --bin ape-cli -- {{args}} bytebeats "{{formula}}"

run-gui:
	cargo run --release --bin ape-gui
