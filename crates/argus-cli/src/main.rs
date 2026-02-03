//! Argus CLI - The Hundred-Eyed Agent

use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io::{self, Write};
use std::path::PathBuf;
use argus_crypto::SecureVault;

const ARGUS_WATCHING: &str = r#"
        ◯ ◯ ◯
       ◯ ◯ ◯ ◯
      ◯  ___  ◯
     ◯  /   \  ◯
    ◯  | ◉ ◉ |  ◯
     ◯  \ ▽ /  ◯
      ◯  |═|  ◯
       ◯/|||\ ◯
        ◯ ◯ ◯"#;

const ARGUS_THINKING: &str = r#"
        ◯ ◯ ◯
       ◯ ◯ ◯ ◯
      ◯  ___  ◯
     ◯  /   \  ◯
    ◯  | ◉ ◉ |  ◯
     ◯  \ ─ /  ◯
      ◯  |≡|  ◯
       ◯/|||\◯
        ◯ ◯ ◯"#;

const LOGO: &str = r#"
    ___    ____  ______  __  _______
   /   |  / __ \/ ____/ / / / / ___/
  / /| | / /_/ / / __  / / / /\__ \ 
 / ___ |/ _, _/ /_/ / / /_/ /___/ / 
/_/  |_/_/ |_|\____/  \____//____/  
"#;

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

struct ChatMessage {
    role: String,
    content: String,
}

struct App {
    messages: Vec<ChatMessage>,
    input: String,
    scroll: u16,
    api_key: String,
    client: reqwest::Client,
    thinking: bool,
}

impl App {
    fn new(api_key: String) -> Self {
        Self {
            messages: vec![],
            input: String::new(),
            scroll: 0,
            api_key,
            client: reqwest::Client::new(),
            thinking: false,
        }
    }

    async fn send_message(&mut self) -> anyhow::Result<()> {
        if self.input.trim().is_empty() {
            return Ok(());
        }

        let user_msg = self.input.clone();
        self.messages.push(ChatMessage {
            role: "You".to_string(),
            content: user_msg.clone(),
        });
        self.input.clear();
        self.thinking = true;

        let resp = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "arcee-ai/trinity-mini:free",
                "messages": [{"role": "user", "content": user_msg}]
            }))
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Error: No response")
            .to_string();

        self.messages.push(ChatMessage {
            role: "Argus".to_string(),
            content,
        });
        self.thinking = false;

        Ok(())
    }
}

async fn run_tui(api_key: String) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(api_key);

    loop {
        terminal.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(22),  // Argus avatar
                    Constraint::Min(40),     // Chat area
                ])
                .split(f.size());

            // Left side - Argus avatar
            let avatar = if app.thinking { ARGUS_THINKING } else { ARGUS_WATCHING };
            let avatar_widget = Paragraph::new(avatar)
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(Span::styled(
                        if app.thinking { " Thinking... " } else { " Watching " },
                        Style::default().fg(if app.thinking { Color::Yellow } else { Color::Cyan })
                    )));
            f.render_widget(avatar_widget, main_chunks[0]);

            // Right side - chat
            let chat_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(10),    // Chat
                    Constraint::Length(3),  // Input
                    Constraint::Length(1),  // Status
                ])
                .split(main_chunks[1]);

            // Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled("👁️  ", Style::default().fg(Color::Cyan)),
                Span::styled("ARGUS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                Span::styled("The Hundred-Eyed Agent", Style::default().fg(Color::DarkGray)),
            ]))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(header, chat_chunks[0]);

            // Chat messages
            let mut chat_lines: Vec<Line> = vec![];
            for msg in &app.messages {
                let (color, prefix) = if msg.role == "You" {
                    (Color::Green, "► ")
                } else {
                    (Color::Cyan, "◉ ")
                };
                chat_lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(color)),
                    Span::styled(&msg.role, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ]));
                for line in msg.content.lines() {
                    chat_lines.push(Line::from(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(if msg.role == "You" { Color::White } else { Color::Gray }),
                    )));
                }
                chat_lines.push(Line::from(""));
            }

            let chat = Paragraph::new(chat_lines)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(Span::styled(" Messages ", Style::default().fg(Color::Cyan))))
                .wrap(Wrap { trim: false })
                .scroll((app.scroll, 0));
            f.render_widget(chat, chat_chunks[1]);

            // Input - always visible
            let input_style = if app.thinking {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            let input = Paragraph::new(app.input.as_str())
                .style(input_style)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if app.thinking { Color::DarkGray } else { Color::Cyan }))
                    .title(Span::styled(
                        if app.thinking { " Wait... " } else { " Message " },
                        Style::default().fg(if app.thinking { Color::Yellow } else { Color::Cyan })
                    )));
            f.render_widget(input, chat_chunks[2]);

            // Status bar
            let status = Paragraph::new(Line::from(vec![
                Span::styled(" ESC", Style::default().fg(Color::Yellow)),
                Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
                Span::styled("ENTER", Style::default().fg(Color::Yellow)),
                Span::styled(" send ", Style::default().fg(Color::DarkGray)),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::styled(" scroll ", Style::default().fg(Color::DarkGray)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("🔐 ", Style::default().fg(Color::Green)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("arcee-ai/trinity-mini", Style::default().fg(Color::Magenta)),
            ]));
            f.render_widget(status, chat_chunks[3]);
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if app.thinking {
                    continue; // Ignore input while thinking
                }
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            app.send_message().await?;
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Up => {
                        app.scroll = app.scroll.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        app.scroll = app.scroll.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("{}", LOGO);
            println!("👁️  Initializing Argus...\n");
            let path = vault_path();
            if path.exists() {
                println!("✅ Vault already exists at {:?}", path);
                return Ok(());
            }
            SecureVault::init(path)?;
            println!("✅ Vault created.");
            println!("✅ Master key stored in system keychain.");
            println!("\n🔐 Your secrets are encrypted. Not plaintext. Not ever.\n");
            println!("Next: argus vault set OPENROUTER_KEY <your-key>");
        }
        Commands::Run => {
            let mut vault = SecureVault::new(vault_path());
            vault.unlock()?;
            
            let api_key = vault.retrieve("OPENROUTER_KEY").map_err(|_| {
                anyhow::anyhow!("No OPENROUTER_KEY found. Run: argus vault set OPENROUTER_KEY")
            })?;

            run_tui(api_key).await?;
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
                    println!("\n🔐 Stored secrets:");
                    for k in vault.list_keys() {
                        println!("   • {}", k);
                    }
                    println!();
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
