# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust application that tests Google Gemini's image analysis capabilities via a third-party API (new-api). The application reads an image file, encodes it to base64, and sends it to the Gemini 3 Pro Image Preview model for analysis.

## Common Development Commands

### Building and Running
```bash
cargo build              # Build the project
cargo run                # Build and run the application
cargo build --release    # Build optimized release version
```

### Testing and Quality
```bash
cargo test               # Run tests
cargo clippy             # Run linter
cargo fmt                # Format code
```

## Architecture

### Core Components

- **src/main.rs**: Single-file application containing:
  - HTTP client setup using `reqwest`
  - Image processing and base64 encoding
  - API request/response structures for OpenAI-compatible format
  - Main async function orchestrating the API call

### Dependencies
- `tokio`: Async runtime with full features
- `reqwest`: HTTP client with JSON support
- `serde`: Serialization/deserialization
- `base64`: Image encoding for API transmission

### API Integration
The application uses OpenAI-compatible API format to communicate with Gemini 3 Pro Image Preview via a custom API endpoint. Request structure includes:
- Model specification: "gemini-3-pro-image-preview"
- Message format with text and image_url content types
- Configurable parameters (max_tokens, temperature)

## Development Notes

### Image Requirements
- The application expects `test.png` in the project root by default
- Supported image formats should be compatible with base64 encoding
- Image path is configurable in `main.rs:65`

### API Configuration
- API endpoint and key are hardcoded in `main.rs:61-62`
- Response is saved to `response.txt` in the project root
- Token usage statistics are displayed in console output

### Error Handling
- File existence validation for images
- HTTP status code checking
- Basic error messages for common failure scenarios