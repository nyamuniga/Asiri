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
    }
}
