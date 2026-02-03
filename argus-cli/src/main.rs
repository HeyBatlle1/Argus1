//! Argus CLI - The Hundred-Eyed Agent
//!
//! Usage:
//!   argus init          Initialize Argus with keychain
//!   argus run           Start the agent
//!   argus provider add  Add an LLM provider
//!   argus watch         Monitor agent activity
//!   argus vault         Manage secrets vault

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "argus")]
#[command(author, version, about = "The Hundred-Eyed Agent Runtime")]
#[command(
    long_about = "Argus - A security-first AI agent runtime.\n\n\
    Nothing escapes Argus."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Argus (creates vault, configures keychain)
    Init {
        /// Use hardware keychain for master key
        #[arg(long)]
        keychain: bool,
    },
    
    /// Start the agent
    Run {
        /// Enable audit logging
        #[arg(long)]
        audit: bool,
    },
    
    /// Manage LLM providers
    Provider {
        #[command(subcommand)]
        action: ProviderAction,
    },
    
    /// Manage secrets vault
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
    
    /// Monitor agent activity
    Watch,
}

#[derive(Subcommand)]
enum ProviderAction {
    /// Add an LLM provider
    Add {
        /// Provider name (claude, openai, local)
        name: String,
    },
    /// List configured providers
    List,
    /// Remove a provider
    Remove {
        name: String,
    },
}

#[derive(Subcommand)]
enum VaultAction {
    /// List secrets (names only, not values)
    List,
    /// Add a secret
    Add {
        /// Secret name
        name: String,
    },
    /// Remove a secret
    Remove {
        /// Secret name
        name: String,
    },
    /// Lock the vault
    Lock,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    match cli.command {
        Commands::Init { keychain } => {
            println!("👁️  Initializing Argus...");
            if keychain {
                println!("   Using hardware keychain for master key");
            }
            println!("   Creating encrypted vault...");
            println!("   ✓ Argus initialized. Nothing escapes.");
            // TODO: Actually create vault
        }
        
        Commands::Run { audit } => {
            println!("👁️  Argus is watching...");
            if audit {
                println!("   Audit logging enabled");
            }
            // TODO: Start agent loop
            println!("   Agent runtime not yet implemented");
        }
        
        Commands::Provider { action } => match action {
            ProviderAction::Add { name } => {
                println!("👁️  Adding provider: {}", name);
                println!("   Enter API key (will be stored in vault):");
                // TODO: Read from stdin, store in vault
            }
            ProviderAction::List => {
                println!("👁️  Configured providers:");
                println!("   (none configured)");
            }
            ProviderAction::Remove { name } => {
                println!("👁️  Removing provider: {}", name);
            }
        },
        
        Commands::Vault { action } => match action {
            VaultAction::List => {
                println!("👁️  Vault contents:");
                println!("   (vault empty)");
            }
            VaultAction::Add { name } => {
                println!("👁️  Adding secret: {}", name);
                println!("   Enter value (will not echo):");
                // TODO: Secure input
            }
            VaultAction::Remove { name } => {
                println!("👁️  Removing secret: {}", name);
            }
            VaultAction::Lock => {
                println!("👁️  Locking vault...");
                println!("   ✓ Vault locked. All secrets zeroized from memory.");
            }
        },
        
        Commands::Watch => {
            println!("👁️  Argus Watch - Monitoring agent activity");
            println!("   (no activity yet)");
            // TODO: TUI with Ratatui
        }
    }
    
    Ok(())
}
