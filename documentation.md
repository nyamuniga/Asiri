# Shamir Secret Sharing (Asiri) — Codebase Walkthrough & Rust Learning Guide

Welcome to the **Shamir Secret Sharing (Asiri)** codebase documentation. This document is designed to help you learn Rust by explaining what every file, function, and line of code is doing in detail.

This project implements **Shamir's Secret Sharing Scheme** over the Galois Field $GF(2^8)$. It allows splitting any sensitive data (such as passwords, keys, or seeds) into $N$ shares, such that any $T$ (threshold) of them can reconstruct the secret, but $T - 1$ shares reveal absolutely nothing.

The project is split into three components:
1. **`core`** (crate name: `asiri-core`): Contains the mathematical logic for Galois Field $GF(2^8)$ arithmetic and the polynomial equations required for Shamir's scheme.
2. **`cli`**: A command-line program that uses `clap` to split or recover secrets via the terminal.
3. **`gui`**: A desktop Tauri application that maps frontend actions to backend Rust logic.

---

## Part 1: Core Mathematics & Scheme (`core/src/`)

Unlike normal algebra, Shamir's Secret Sharing must be performed over a **Finite Field** (or Galois Field) so that values do not overflow or leak fractional information. The chosen field is $GF(2^8)$, which contains exactly 256 elements (fitting perfectly in a single byte `u8`).

### 1. `gf256.rs`

This file implements the mathematical foundations of the Galois Field $GF(2^8)$ using the irreducible reduction polynomial $x^8 + x^4 + x^3 + x + 1$ (commonly represented as `0x11b` or `0x1b`).

```rust
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use zeroize::Zeroize;

/// Element in the Galois Field GF(2^8).
/// Polynomial: x^8 + x^4 + x^3 + x + 1 (0x11B)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Zeroize)]
pub struct Gf256(pub u8);
```

#### Line-by-Line Explanation:

- **Line 1: `use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};`**
  Imports standard library operator traits. In Rust, to use math operators like `+`, `-`, or `*` on custom structs, you must implement these traits.
- **Line 2: `use zeroize::Zeroize;`**
  Imports the `Zeroize` trait, which allows us to securely overwrite variables in memory with zeros when we are done using them.
- **Line 6: `#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Zeroize)]`**
  An attribute macro that automatically generates implementations for standard traits:
  - `Clone` and `Copy`: Allows instances to be copied by value rather than moved.
  - `Debug`: Enables formatting with `{:?}`.
  - `PartialEq` and `Eq`: Allows comparison using `==` and `!=`.
  - `Default`: Initializes the struct to `Gf256(0)`.
  - `Zeroize`: Enables secure deletion.
- **Line 7: `pub struct Gf256(pub u8);`**
  Defines a public **Tuple Struct** containing a single public field of type `u8`. This is known as the *Newtype Pattern* in Rust, which wraps a primitive type in a custom type to enforce compiler type checks.

---

#### Multiplicative Inverse inside `impl Gf256`:
In finite fields, division is performed by multiplying by the multiplicative inverse. For any non-zero element $a$, there exists an $a^{-1}$ such that $a \cdot a^{-1} = 1$.

```rust
    /// Computes the multiplicative inverse of `self`.
    /// Panics if `self` is zero.
    pub fn inverse(self) -> Self {
        assert_ne!(self.0, 0, "Zero has no inverse in GF(2^8)");
        // a^254 = a^-1 in GF(2^8)
        let mut res = Gf256::ONE;
        let mut base = self;
        let mut exp = 254u8;
        while exp > 0 {
            if exp & 1 == 1 {
                res = res * base;
            }
            base = base * base;
            exp >>= 1;
        }
        res
    }
```

- **Line 15: `pub fn inverse(self) -> Self {`**
  Declares a public method on `Gf256` that consumes `self` (since it is a cheap `Copy` type) and returns a new `Gf256`.
- **Line 16: `assert_ne!(self.0, 0, "Zero has no inverse in GF(2^8)");`**
  A macro that checks that the inner value is not zero. If it is 0, the thread panics and terminates with the error message.
- **Line 17: `// a^254 = a^-1 in GF(2^8)`**
  In a finite field of size $q = 2^8$, Fermat's Little Theorem states that for any non-zero element $a$, $a^{q-1} \equiv 1 \pmod q$, meaning $a^{255} = 1$. Thus, $a \cdot a^{254} = 1$, which proves that the inverse $a^{-1}$ is equal to $a^{254}$. We compute this power using binary exponentiation (exponentiation by squaring).
- **Line 18: `let mut res = Gf256::ONE;`**
  Initializes the accumulator variable `res` with the field element $1$.
- **Line 19: `let mut base = self;`**
  Creates a mutable copy of `self` to represent base powers ($a, a^2, a^4, a^8$, etc.).
- **Line 20: `let mut exp = 254u8;`**
  The exponent we want to raise the base to ($254$).
- **Line 21: `while exp > 0 {`**
  Loops as long as there are bits remaining to process in the exponent.
- **Line 22: `if exp & 1 == 1 {`**
  Checks if the lowest bit of the exponent is set using bitwise AND.
- **Line 23: `res = res * base;`**
  If the bit is set, multiplies the accumulator by the current base power. Note that this uses the custom multiplication operator we implement below.
- **Line 25: `base = base * base;`**
  Squares the base power for the next bit position.
- **Line 26: `exp >>= 1;`**
  Shifts the exponent right by 1 bit.
- **Line 28: `res`**
  Returns the final result $a^{254}$.

---

#### Polynomial Evaluation:

```rust
    /// Evaluates a polynomial (where `coeffs[0]` is the constant term) at `x = self`.
    pub fn evaluate_polynomial(coeffs: &[Gf256], x: Gf256) -> Gf256 {
        // Horner's method
        let mut result = Gf256::ZERO;
        for &coeff in coeffs.iter().rev() {
            result = (result * x) + coeff;
        }
        result
    }
```

- **Line 32: `pub fn evaluate_polynomial(coeffs: &[Gf256], x: Gf256) -> Gf256 {`**
  Evaluates a polynomial $P(x) = c_0 + c_1 x + c_2 x^2 + \dots$ at coordinate $x$.
- **Line 34: `let mut result = Gf256::ZERO;`**
  Initializes the running evaluation sum to zero.
- **Line 35: `for &coeff in coeffs.iter().rev() {`**
  Iterates over the coefficients list in reverse order (from highest degree coefficient to lowest). `rev()` is an iterator modifier.
- **Line 36: `result = (result * x) + coeff;`**
  Applies Horner's Method: $P(x) = ((c_n x + c_{n-1})x + c_{n-2})x + \dots + c_0$. This avoids calculating powers of $x$ manually, making evaluation highly efficient.

---

#### Addition and Subtraction (`impl Add / Sub for Gf256`):
In binary finite fields, addition and subtraction are identical and correspond to the bitwise **XOR** operator (`^`).

```rust
impl Add for Gf256 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Gf256(self.0 ^ rhs.0)
    }
}

impl Sub for Gf256 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs
    }
}
```

- **Line 44: `impl Add for Gf256 {`**
  Implements addition trait.
- **Line 46: `#[inline]`**
  A hint to the compiler to paste the body of this function directly into the call site to eliminate function call overhead.
- **Line 48: `Gf256(self.0 ^ rhs.0)`**
  Addition is implemented as the bitwise XOR of the internal `u8` values. For example, $5 + 3$ in $GF(2^8)$ is `0b101 ^ 0b011 = 0b110` ($6$).
- **Line 62: `self + rhs`**
  Since subtraction is identical to addition in fields of characteristic 2 (where $+1 = -1$), subtraction is implemented by calling addition.

---

#### Multiplication (`impl Mul for Gf256`):
Multiplication in $GF(2^8)$ is polynomial multiplication modulo the irreducible reduction polynomial $x^8 + x^4 + x^3 + x + 1$ (represented as `0x1B` after discarding the 9th overflow bit).

```rust
impl Mul for Gf256 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut a = self.0;
        let mut b = rhs.0;
        let mut p = 0u8;

        for _ in 0..8 {
            if b & 1 == 1 {
                p ^= a;
            }
            let hi_bit_set = a & 0x80;
            a <<= 1;
            if hi_bit_set == 0x80 {
                a ^= 0x1b; // Reduction polynomial
            }
            b >>= 1;
        }
        Gf256(p)
    }
}
```

- **Line 76: `fn mul(self, rhs: Self) -> Self::Output {`**
  Defines multiplication.
- **Lines 77–79: `let mut a = self.0; let mut b = rhs.0; let mut p = 0u8;`**
  Initializes local variable copies of inputs and an accumulator `p` to zero.
- **Line 81: `for _ in 0..8 {`**
  Iterates over each of the 8 bits of multiplier `b` (Russian Peasant Multiplication).
- **Line 82: `if b & 1 == 1 {`**
  If the lowest bit of `b` is 1, adds current multiplicand `a` to the product accumulator `p` using XOR (`^=`).
- **Line 85: `let hi_bit_set = a & 0x80;`**
  Checks if the highest bit (MSB) of `a` is set (`0x80` is `0b10000000`).
- **Line 86: `a <<= 1;`**
  Shifts `a` left by 1 (equivalent to multiplying polynomial $a(x)$ by $x$).
- **Line 87: `if hi_bit_set == 0x80 {`**
  If the highest bit was set prior to shifting, the value has overflowed the 8-bit boundary (degree 8 polynomial).
- **Line 88: `a ^= 0x1b;`**
  Performs modulo reduction by XORing with the lower 8 bits of the reduction polynomial (`0x11B`, so we XOR with `0x1B`).
- **Line 90: `b >>= 1;`**
  Shifts `b` right by 1 to process the next bit.
- **Line 92: `Gf256(p)`**
  Returns the final accumulated product.

---

### 2. `shamir.rs`

This file implements Shamir's Secret Sharing scheme: splitting secrets using random polynomials and reconstructing secrets via Lagrange interpolation.

```rust
use crate::gf256::Gf256;
use rand::Rng;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};
use std::collections::HashSet;

#[derive(Error, Debug)]
pub enum ShamirError {
    #[error("Threshold must be between 2 and 255")]
    InvalidThreshold,
    #[error("Total shares must be >= threshold and <= 255")]
    InvalidTotalShares,
    #[error("Secret cannot be empty")]
    EmptySecret,
    #[error("Duplicate share indices provided")]
    DuplicateShares,
    #[error("Inconsistent share lengths")]
    InconsistentShareLengths,
}
```

#### Line-by-Line Explanation:
- **Line 3: `use thiserror::Error;`**
  Imports the `Error` macro from the `thiserror` crate, which generates boiler-plate implementation code for standard error traits in Rust.
- **Lines 7–19: `pub enum ShamirError { ... }`**
  Declares the errors that can occur during split or recovery actions. The attribute `#[error(...)]` specifies the string description formatting for each enum variant.
- **Lines 21–25: `pub struct Share { pub index: u8, pub data: Vec<u8> }`**
  Represents a single share. `index` is the $x$-coordinate on the polynomial ($1 \dots N$). `data` contains the $y$-coordinates.
  The macro `#[derive(Zeroize, ZeroizeOnDrop)]` ensures that when this share is dropped from memory, the vector's sensitive data is securely zeroed out.

---

#### Splitting Secrets (`split_secret`):

```rust
/// Splits a secret into `total_shares`, requiring `threshold` shares to reconstruct.
pub fn split_secret(secret: &[u8], threshold: u8, total_shares: u8) -> Result<Vec<Share>, ShamirError> {
    if threshold < 2 { return Err(ShamirError::InvalidThreshold); }
    if total_shares < threshold { return Err(ShamirError::InvalidTotalShares); }
    if secret.is_empty() { return Err(ShamirError::EmptySecret); }

    let mut rng = rand::thread_rng();
    let mut shares = vec![
        Share { index: 0, data: vec![0; secret.len()] }; 
        total_shares as usize
    ];

    for i in 0..total_shares {
        shares[i as usize].index = i + 1;
    }

    // Process byte by byte to support arbitrary length secrets
    for byte_idx in 0..secret.len() {
        let secret_byte = Gf256(secret[byte_idx]);
        
        // Generate random coefficients. coeffs[0] is the secret byte.
        let mut coeffs: Zeroizing<Vec<Gf256>> = Zeroizing::new(vec![Gf256::ZERO; threshold as usize]);
        coeffs[0] = secret_byte;
        for i in 1..threshold as usize {
            coeffs[i] = Gf256(rng.gen::<u8>());
        }

        // Evaluate the polynomial for each share
        for i in 0..total_shares as usize {
            let x = Gf256(shares[i].index);
            let y = Gf256::evaluate_polynomial(&coeffs, x);
            shares[i].data[byte_idx] = y.0;
        }
    }

    Ok(shares)
}
```

- **Line 28: `pub fn split_secret(...) -> Result<Vec<Share>, ShamirError> {`**
  Takes a secret byte slice `&[u8]`, a `threshold` ($T$), and `total_shares` ($N$). Returns `Ok` with the list of shares or `Err(ShamirError)`.
- **Lines 29–31: validation guards**
  Validates that threshold is at least 2, total shares is greater than or equal to threshold, and the secret is not empty.
- **Line 33: `let mut rng = rand::thread_rng();`**
  Retrieves a thread-local random number generator to generate polynomial coefficients.
- **Lines 34–37: `let mut shares = vec![ ... ];`**
  Allocates the list of $N$ shares, initializing their data buffers to match the secret's length.
- **Lines 39–41: `for i in 0..total_shares { shares[i as usize].index = i + 1; }`**
  Sets the $x$-coordinate index of each share. We use `i + 1` so that index `0` is excluded. In Shamir's scheme, $P(0)$ is the secret itself, so we must never distribute a share with $x=0$.
- **Line 44: `for byte_idx in 0..secret.len() {`**
  We split the secret byte-by-byte. Each byte index has its own independent random polynomial.
- **Line 45: `let secret_byte = Gf256(secret[byte_idx]);`**
  Converts the current secret byte to a field element.
- **Line 48: `let mut coeffs: Zeroizing<Vec<Gf256>> = Zeroizing::new(vec![Gf256::ZERO; threshold as usize]);`**
  Allocates a temporary vector of coefficients of size `threshold` ($T$) inside a `Zeroizing` wrapper to protect random polynomial variables.
- **Line 49: `coeffs[0] = secret_byte;`**
  Sets the constant term of the polynomial $c_0$ to the secret byte ($P(0) = c_0$).
- **Lines 50–52: `for i in 1..threshold as usize { coeffs[i] = Gf256(rng.gen::<u8>()); }`**
  Fills the remaining $T - 1$ coefficients with random bytes. This creates a random polynomial of degree $T - 1$.
- **Line 55: `for i in 0..total_shares as usize {`**
  Loops through each of the $N$ shares.
- **Line 56: `let x = Gf256(shares[i].index);`**
  The share's $x$-coordinate index.
- **Line 57: `let y = Gf256::evaluate_polynomial(&coeffs, x);`**
  Evaluates the random polynomial at $x$.
- **Line 58: `shares[i].data[byte_idx] = y.0;`**
  Stores the evaluation result $y$ inside the share's data buffer at the current byte index.
- **Line 62: `Ok(shares)`**
  Returns the complete list of generated shares.

---

#### Reconstructing Secrets (`recover_secret`):

To reconstruct the secret, we use **Lagrange Interpolation**. Given $T$ coordinates $(x_i, y_i)$, we calculate $P(0)$ using the formula:
$$P(0) = \sum_{i=1}^{T} y_i \prod_{j \neq i} \frac{x_j}{x_j - x_i}$$

```rust
/// Recovers the original secret from a slice of shares using Lagrange Interpolation.
pub fn recover_secret(shares: &[Share]) -> Result<Zeroizing<Vec<u8>>, ShamirError> {
    if shares.is_empty() { return Err(ShamirError::EmptySecret); }
    let secret_len = shares[0].data.len();
    if secret_len == 0 { return Err(ShamirError::EmptySecret); }

    let mut indices = HashSet::new();
    for s in shares {
        if s.data.len() != secret_len { return Err(ShamirError::InconsistentShareLengths); }
        if !indices.insert(s.index) { return Err(ShamirError::DuplicateShares); }
    }

    let mut secret = Zeroizing::new(vec![0u8; secret_len]);

    // Recover byte by byte
    for byte_idx in 0..secret_len {
        let mut recovered = Gf256::ZERO;
        
        for (i, share_i) in shares.iter().enumerate() {
            let x_i = Gf256(share_i.index);
            let y_i = Gf256(share_i.data[byte_idx]);
            
            let mut num = Gf256::ONE;
            let mut den = Gf256::ONE;
            
            // Calculate the Lagrange basis polynomial l_i(0)
            for (j, share_j) in shares.iter().enumerate() {
                if i == j { continue; }
                let x_j = Gf256(share_j.index);
                num = num * x_j;
                den = den * (x_j - x_i);
            }
            
            let term = y_i * num * den.inverse();
            recovered = recovered + term;
        }
        
        secret[byte_idx] = recovered.0;
    }

    Ok(secret)
}
```

- **Lines 67–75: validation checks**
  Validates that shares list is not empty, check that all shares have identical lengths, and uses a `HashSet` to ensure no two shares have the same $x$-coordinate index.
- **Line 77: `let mut secret = Zeroizing::new(vec![0u8; secret_len]);`**
  Allocates a `Zeroizing` buffer to store the recovered secret.
- **Line 80: `for byte_idx in 0..secret_len {`**
  Iterates over each byte index to reconstruct the secret byte-by-byte.
- **Line 81: `let mut recovered = Gf256::ZERO;`**
  Accumulator for the final byte sum.
- **Line 83: `for (i, share_i) in shares.iter().enumerate() {`**
  Iterates over each share $i$.
- **Lines 84–85: `let x_i = Gf256(share_i.index); let y_i = Gf256(share_i.data[byte_idx]);`**
  Loads the coordinates $(x_i, y_i)$.
- **Lines 87–88: `let mut num = Gf256::ONE; let mut den = Gf256::ONE;`**
  Initializes numerator and denominator accumulators to $1$ for computing the Lagrange basis coefficient:
  $$l_i(0) = \prod_{j \neq i} \frac{x_j}{x_j - x_i}$$
- **Line 91: `for (j, share_j) in shares.iter().enumerate() {`**
  Iterates over all other shares $j$.
- **Line 92: `if i == j { continue; }`**
  Skips calculations when comparing a share with itself to avoid division by zero ($x_i - x_i$).
- **Line 93: `let x_j = Gf256(share_j.index);`**
  Loads the coordinate $x_j$.
- **Line 94: `num = num * x_j;`**
  Multiplies the numerator by $x_j$ (since evaluating at $0$ reduces $(x - x_j)$ to $-x_j$, which is equivalent to $+x_j$ in $GF(2^8)$).
- **Line 95: `den = den * (x_j - x_i);`**
  Multiplies the denominator by $(x_j - x_i)$.
- **Line 98: `let term = y_i * num * den.inverse();`**
  Calculates the term: $y_i \cdot \frac{\text{numerator}}{\text{denominator}}$. Division is performed by multiplying by `den.inverse()`.
- **Line 99: `recovered = recovered + term;`**
  Adds the term to our running summation (using XOR).
- **Line 102: `secret[byte_idx] = recovered.0;`**
  Saves the reconstructed byte.
- **Line 105: `Ok(secret)`**
  Returns the reconstructed secret wrapped in the `Zeroizing` container.

---

## Part 2: Command Line Interface (`cli/src/`)

The CLI app wraps the core library to allow command-line split/recovery actions.

### 1. `main.rs`

Uses `clap` to parse arguments and handles hidden inputs for security.

```rust
use asiri_core::{split_secret, recover_secret, Share};
use clap::{Parser, Subcommand};
use rpassword::read_password;
use std::io::{self, Write};
use zeroize::Zeroize;

#[derive(Parser)]
#[command(name = "asiri")]
#[command(about = "Advanced Shamir Secret Sharing (Asiri)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Split a secret into multiple shares
    Split {
        /// Number of shares required to recover the secret
        #[arg(short, long)]
        threshold: u8,
        /// Total number of shares to generate
        #[arg(short, long)]
        shares: u8,
    },
    /// Recover a secret from shares
    Recover,
}
```

#### Line-by-Line Explanation:
- **Lines 7–13: `struct Cli { ... }`**
  Defines the root parser structure for command-line arguments. The `#[derive(Parser)]` macro handles generating the command-line interface.
- **Lines 15–28: `enum Commands { ... }`**
  Defines subcommands: `Split` (accepts options `-t` / `--threshold` and `-s` / `--shares`) and `Recover`.

---

#### The Execution Flow in `main`:

```rust
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Split { threshold, shares } => {
            println!("Enter the secret to split (input is hidden): ");
            let mut secret = read_password().expect("Failed to read password");
            
            match split_secret(secret.as_bytes(), threshold, shares) {
                Ok(generated_shares) => {
                    println!("\nSuccessfully generated {} shares. Keep them safe!", generated_shares.len());
                    for share in generated_shares {
                        // Output in format: <index>-<hex_data>
                        let hex_data = hex::encode(&share.data);
                        println!("{}-{}", share.index, hex_data);
                    }
                },
                Err(e) => eprintln!("Error splitting secret: {}", e),
            }
            secret.zeroize();
        }
```

- **Line 31: `let cli = Cli::parse();`**
  Parses command line arguments passed to the program.
- **Line 33: `match cli.command {`**
  Uses pattern matching to determine which subcommand was called.
- **Line 36: `let mut secret = read_password().expect("Failed to read password");`**
  Reads input from stdin with echoing disabled using `rpassword`, keeping the secret hidden on screen as it is typed.
- **Line 38: `match split_secret(secret.as_bytes(), threshold, shares) {`**
  Passes the secret bytes to the core library split function.
- **Lines 41–45: `for share in generated_shares { ... }`**
  Prints the generated shares to stdout under the format `<index>-<hex_payload>`. E.g. `1-a5f2c3`.
- **Line 49: `secret.zeroize();`**
  Overwrites the secret string memory with zeros before exiting.

#### Recovery execution block:

```rust
        Commands::Recover => {
            println!("Enter shares one by one (format: <index>-<hex_data>). Leave blank to finish:");
            let mut collected_shares = Vec::new();
            
            loop {
                print!("> ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                
                if input.is_empty() {
                    break;
                }
                
                let parts: Vec<&str> = input.splitn(2, '-').collect();
                if parts.len() != 2 {
                    eprintln!("Invalid format. Use <index>-<hex_data>");
                    continue;
                }
                
                let index = match parts[0].parse::<u8>() {
                    Ok(i) => i,
                    Err(_) => {
                        eprintln!("Invalid index.");
                        continue;
                    }
                };
                
                let data = match hex::decode(parts[1]) {
                    Ok(d) => d,
                    Err(_) => {
                        eprintln!("Invalid hex data.");
                        continue;
                    }
                };
                
                collected_shares.push(Share { index, data });
            }
            
            match recover_secret(&collected_shares) {
                Ok(secret) => {
                    if let Ok(string_secret) = String::from_utf8(secret.to_vec()) {
                        println!("\nRecovered Secret: {}", string_secret);
                    } else {
                        println!("\nRecovered Secret (Hex): {}", hex::encode(&*secret));
                    }
                },
                Err(e) => eprintln!("\nFailed to recover secret: {}", e),
            }
        }
```

- **Line 53: `let mut collected_shares = Vec::new();`**
  Initializes a vector to store the user-entered shares.
- **Line 55: `loop { ... }`**
  An infinite loop that continually prompts the user for shares.
- **Line 62: `if input.is_empty() { break; }`**
  Breaks out of the loop when the user enters an empty line.
- **Line 66: `let parts: Vec<&str> = input.splitn(2, '-').collect();`**
  Splits the input on the first `-` character to separate index from hex data.
- **Line 72: `let index = match parts[0].parse::<u8>() { ... }`**
  Parses the index string into a `u8` integer.
- **Line 80: `let data = match hex::decode(parts[1]) { ... }`**
  Decodes the hex data back into bytes.
- **Line 88: `collected_shares.push(Share { index, data });`**
  Builds a `Share` struct and appends it to the collection.
- **Line 91: `match recover_secret(&collected_shares) {`**
  Calls the core library reconstruction function.
- **Line 93: `if let Ok(string_secret) = String::from_utf8(secret.to_vec()) {`**
  Attempts to parse the recovered bytes as a UTF-8 text string. If successful, it prints the string. If not (e.g. if the secret was raw binary data), it prints the hex representation of the bytes instead.

---

## Part 3: Tauri Desktop App Wrapper (`gui/src-tauri/src/`)

The GUI app contains command endpoints exposed to a web frontend.

### 1. `lib.rs`

Exposes Tauri command handlers for splitting and recovering secrets.

```rust
use asiri_core::{split_secret, recover_secret, Share};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Serialize, Deserialize)]
pub struct ShareDto {
    pub index: u8,
    pub data: String, // Hex encoded
}
```

- **Lines 5–9: `ShareDto`**
  Data Transfer Object (DTO) struct used for passing share data between Rust and JavaScript. The web frontend receives and sends share payloads as hex-encoded strings rather than raw byte arrays.

---

#### Tauri Commands:

```rust
#[tauri::command]
fn split_secret_cmd(mut secret: String, threshold: u8, shares: u8) -> Result<Vec<ShareDto>, String> {
    let result = match split_secret(secret.as_bytes(), threshold, shares) {
        Ok(generated_shares) => {
            let dtos = generated_shares.into_iter().map(|s| ShareDto {
                index: s.index,
                data: hex::encode(&s.data),
            }).collect();
            Ok(dtos)
        }
        Err(e) => Err(e.to_string()),
    };
    secret.zeroize();
    result
}
```

- **Line 11: `#[tauri::command]`**
  Exposes the function to JS/TS invocation via Tauri's IPC bridge.
- **Line 13: `match split_secret(secret.as_bytes(), threshold, shares)`**
  Invokes the core library split function.
- **Lines 15–18: `generated_shares.into_iter().map(...).collect()`**
  Maps each core `Share` struct into a `ShareDto` by hex-encoding the byte arrays, compiling them into a vector.
- **Line 23: `secret.zeroize();`**
  Zeroes out the temporary parameter variable containing the secret string.

#### Recovery Command:

```rust
#[tauri::command]
fn recover_secret_cmd(shares: Vec<ShareDto>) -> Result<String, String> {
    let mut core_shares = Vec::new();
    for dto in shares {
        let data = hex::decode(&dto.data).map_err(|e| format!("Invalid hex: {}", e))?;
        core_shares.push(Share { index: dto.index, data });
    }

    match recover_secret(&core_shares) {
        Ok(secret_bytes) => {
            // Try to parse as UTF-8 string
            match String::from_utf8(secret_bytes.to_vec()) {
                Ok(s) => Ok(s),
                Err(_) => Ok(hex::encode(&*secret_bytes)), // Fallback to hex
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
```

- **Lines 29–33: DTO mapping loop**
  Iterates over the input `ShareDto` list, hex-decodes the payloads, and instantiates core `Share` structs.
- **Line 35: `match recover_secret(&core_shares) {`**
  Calls the core library reconstruction function.
- **Lines 38–41: String resolution**
  Resolves the result bytes as a readable UTF-8 string, falling back to hex encoding if the bytes are raw binary.

### 2. `main.rs` & `build.rs`
- **`main.rs`**: Passes control to the application library run command: `Asiri::run()`.
- **`build.rs`**: Compiles resources using `tauri_build::build()`.
