.PHONY: all build start run clean proposal

all: build

build: proposal
	cargo build --release

proposal: proposal.typ
	typst c proposal.typ

start:
	target/release/project5

run: start

clean:
	cargo clean
	rm -f proposal.pdf
