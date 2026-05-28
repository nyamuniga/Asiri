<p align="center">
  <img src="gui/src/assets/logo.png" width="200" alt="Asiri Logo">
</p>

# Asiri (Advanced Shamir Secret Sharing)

Asiri is an advanced, production-grade cryptographic tool for securely splitting and recovering sensitive secrets using Shamir's Secret Sharing (SSS) over Galois Field GF(2^8).

"Asiri" means "secret" in Yoruba, and the project is designed with a premium cypherpunk African-inspired aesthetic.

## Architecture

This project is a Cargo Workspace divided into three components:

1. **asiri-core**: A pure Rust cryptography library implementing GF(2^8) math and Lagrange interpolation. It leverages the `zeroize` crate to ensure that secrets are wiped from RAM immediately after processing.
2. **asiri-cli**: A robust command-line interface for advanced users to script and automate secret sharing securely from the terminal.
3. **gui (Tauri + React)**: A stunning, cross-platform graphical user interface featuring dark mode, neon aesthetics, and traditional African geometric patterns.

## Building and Installation

### CLI Installation (Global)
To install the `asiri` command globally on your machine:
```bash
cargo install --path path-to-cli-folder
```
Once installed, you can run the CLI from anywhere:
```bash
asiri split -t 3 -n 5
asiri recover
```

### GUI Installation
To run the Cypherpunk UI in development mode:
```bash
cd gui
npm install
npm run tauri dev
```

To build the final, optimized desktop application (e.g., a `.dmg` or `.app` on Mac, `.exe` on Windows):
```bash
cd gui
npm run tauri build
```
The compiled executable will be located inside the `gui/src-tauri/target/release/bundle` directory!

## Security
- Uses industry standard GF(256) arithmetic, preventing modulo biases found in simple prime implementations.
- Zero-dependency core logic (besides `rand` and `zeroize`).
- All intermediate buffers (and the plaintext secrets themselves) are eagerly zeroized.

