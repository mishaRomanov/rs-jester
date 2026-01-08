help:
	@echo "Makefile commands:"
	@echo "  setup_mac          - Install Rust and Go on macOS using Homebrew"
	@echo "  setup_linux        - Install Rust and Go on Linux using apt"
	@echo "  run                - Build and run the Rust application"
	@echo "  run_mock_upstream  - Run the mock upstream server written in Go"

setup_mac:	
	brew install rust
	brew install go 

# AI-Generated, not tested.
setup_linux:
	sudo apt update
	sudo apt install -y curl build-essential
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	source $HOME/.cargo/env
	sudo apt install -y golang 

run: 
	cargo run

run_mock_upstream:
	go run ./bexmpl/main.go
