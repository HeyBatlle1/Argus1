//! Argus CLI

use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;
use argus_crypto::SecureVault;

#[derive(Parser)]
#[command(name = "argus", version, about = "The hundred-eyed agent runtime")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
}

#[derive(Subcommand)]
enum VaultAction {
    Set { key: String },
    Get { key: String },
    List,
    Delete { key: String },
}

fn vault_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".argus").join("vault.enc")
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("👁️  Initializing Argus...");
            let path = vault_path();
            if path.exists() {
                println!("Vault already exists at {:?}", path);
                return Ok(());
            }
            SecureVault::init(path)?;
            println!("✅ Vault created. Master key stored in system keychain.");
        }
        Commands::Vault { action } => {
            let mut vault = SecureVault::new(vault_path());
            vault.unlock()?;
            
            match action {
                VaultAction::Set { key } => {
                    print!("Enter secret value: ");
                    io::stdout().flush()?;
                    let mut value = String::new();
                    io::stdin().read_line(&mut value)?;
                    vault.store(&key, value.trim())?;
                    println!("✅ Stored: {}", key);
                }
                VaultAction::Get { key } => {
                    match vault.retrieve(&key) {
                        Ok(v) => println!("{}", v),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                VaultAction::List => {
                    for k in vault.list_keys() {
                        println!("  {}", k);
                    }
                }
                VaultAction::Delete { key } => {
                    vault.delete(&key)?;
                    println!("✅ Deleted: {}", key);
                }
            }
        }
    }
    Ok(())
}
