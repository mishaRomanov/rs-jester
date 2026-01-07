help:
	@echo "Available commands:"
	@echo "  build             Build the Rust project"
	@echo "  run               Run the Rust project"
	@echo "  run_mock_upstream Run the mock upstream server"

run: 
	cargo run

run_mock_upstream:
	go run ./bexmpl/main.go
