.PHONY: all client test clean

all: client

test:
	cargo build --release
	cp target/release/4700send .
	cp target/release/4700recv .


client:
	# @which cargo > /dev/null || apt install -y curl
	# curl https://sh.rustup.rs -sSf | sh -s -- -y
	# . "$HOME/.cargo/env"
	apt update
	apt install cargo -y
	cargo build --release
	cp target/release/4700send .
	cp target/release/4700recv .

clean:
	cargo clean
	rm -f 4700send
	rm -f 4700recv
