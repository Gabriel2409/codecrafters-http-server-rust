.PHONY: help curl

# Display help message
help:
	@echo "Available targets:"
	@echo "  - help: Display this help message."
	@echo "  - curl: Runs a request including headers to our server"


curl:
	curl -i http://localhost:4221

