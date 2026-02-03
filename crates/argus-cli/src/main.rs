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
    Run,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        Commands::Run => {
            let mut vault = SecureVault::new(vault_path());
            vault.unlock()?;
            
            let api_key = vault.retrieve("OPENROUTER_KEY").map_err(|_| {
                anyhow::anyhow!("No OPENROUTER_KEY found. Run: argus vault set OPENROUTER_KEY")
            })?;

            println!("👁️  Argus is watching. Type 'exit' to quit.\n");
            
            let client = reqwest::Client::new();
            
            loop {
                print!("You: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                
                if input == "exit" || input == "quit" {
                    println!("👁️  Argus sleeps.");
                    break;
                }

                let resp = client.post("https://openrouter.ai/api/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&serde_json::json!({
                        "model": "z-ai/glm-4.5-air:free",
                        "messages": [{"role": "user", "content": input}]
                    }))
                    .send()
                    .await?;

                let json: serde_json::Value = resp.json().await?;
                if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                    println!("\nArgus: {}\n", content);
                } else {
                    println!("\nError: {:?}\n", json);
                }
            }
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
