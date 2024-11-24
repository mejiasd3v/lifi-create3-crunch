# LiFi CREATE3 Crunch

A high-performance command-line tool for generating vanity addresses using the CREATE3 deployment method. This tool helps you find specific salt values that will result in contract addresses with desired patterns.

## Features

- Generate contract addresses with custom prefixes and/or suffixes
- Parallel processing support for faster address generation using all available CPU cores
- Real-time progress tracking and detailed output
- Configurable maximum attempts to control search duration
- Silent mode for programmatic usage and CI/CD integration
- Memory-efficient implementation with minimal allocations
- Support for case-insensitive pattern matching

## How It Works

The tool uses the CREATE3 deployment method to generate deterministic contract addresses. It works by:

1. Generating random salt values
2. Computing the resulting contract address using the CREATE3 formula
3. Checking if the generated address matches the desired pattern
4. Using parallel processing to leverage multiple CPU cores for faster results

## Installation

### Prerequisites

1. Ensure you have Rust installed on your system (version 1.70.0 or later)
   - Install from [rustup.rs](https://rustup.rs/)
   - Verify installation: `rustc --version`

### Building from Source

1. Clone the repository:
    ```bash
    git clone <repository-url>
    cd lifi-create3-crunch
    ```

2. Build the release version:
    ```bash
    cargo build --release
    ```

3. The binary will be available at `./target/release/lifi-create3-crunch`

## Usage

### Basic Command Structure
```bash
lifi-create3-crunch [OPTIONS] --creator <CREATOR_ADDRESS>
```

### Options

- `-c, --creator <ADDRESS>`: The creator address (required)
- `-s, --starts-with <PREFIX>`: Desired address prefix (without 0x)
- `-e, --ends-with <SUFFIX>`: Desired address suffix
- `-z, --leading-zeros <NUMBER>`: Number of leading zeros in the address
- `--silent`: Run in silent mode (no progress output)
- `-p, --parallel`: Enable parallel processing for faster generation

### Examples

Find an address starting with "dead":
```bash
lifi-create3-crunch -c 0x742d35Cc6634C0532925a3b844Bc454e4438f44e -s dead
```

Find an address ending with "beef":
```bash
lifi-create3-crunch -c 0x742d35Cc6634C0532925a3b844Bc454e4438f44e -e beef
```

Find an address with both prefix and suffix using parallel processing:
```bash
lifi-create3-crunch -c 0x742d35Cc6634C0532925a3b844Bc454e4438f44e -s dead -e beef -p
```
