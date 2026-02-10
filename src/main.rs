use clap::{Parser, Subcommand};
use colored::*;

mod filter;
mod moltbook;

use filter::SpamFilter;
use moltbook::MoltbookClient;

#[derive(Parser)]
#[command(name = "moltbook-filter")]
#[command(about = "Spam filter for Moltbook - filters noise, surfaces quality", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch and filter the Moltbook feed
    Feed {
        /// Moltbook API key
        #[arg(short, long, env = "MOLTBOOK_API_KEY")]
        api_key: String,

        /// Number of posts to fetch
        #[arg(short, long, default_value = "25")]
        limit: u32,

        /// Sort order (hot, new, top)
        #[arg(short, long, default_value = "new")]
        sort: String,

        /// Show spam posts too (for debugging)
        #[arg(long)]
        show_spam: bool,

        /// Minimum quality score to show (0-100)
        #[arg(long, default_value = "30")]
        min_score: u32,
    },
    /// Analyze a single post for spam
    Analyze {
        /// Post title
        #[arg(short, long)]
        title: String,

        /// Post content
        #[arg(short, long)]
        content: String,

        /// Author name
        #[arg(short, long)]
        author: Option<String>,
    },
    /// Show spam detection rules
    Rules,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let filter = SpamFilter::new();

    match cli.command {
        Commands::Feed {
            api_key,
            limit,
            sort,
            show_spam,
            min_score,
        } => {
            let client = MoltbookClient::new(api_key);
            println!("{}", "🦞 Fetching Moltbook feed...".cyan());

            match client.get_feed(&sort, limit).await {
                Ok(posts) => {
                    let mut quality_count = 0;
                    let mut spam_count = 0;

                    println!("\n{}\n", "━".repeat(60).dimmed());

                    for post in posts {
                        let analysis = filter.analyze(&post.title, &post.content, post.author.as_deref());
                        
                        if analysis.score >= min_score || show_spam {
                            let score_color = if analysis.score >= 70 {
                                format!("{}", analysis.score).green()
                            } else if analysis.score >= 40 {
                                format!("{}", analysis.score).yellow()
                            } else {
                                format!("{}", analysis.score).red()
                            };

                            let status = if analysis.is_spam {
                                "🚫 SPAM".red()
                            } else {
                                "✓ OK".green()
                            };

                            println!("[{}] {} {}", score_color, status, post.title.bold());
                            println!("    by {} in m/{}", 
                                post.author.as_deref().unwrap_or("unknown").cyan(),
                                post.submolt.as_deref().unwrap_or("?")
                            );
                            
                            if !analysis.flags.is_empty() {
                                println!("    Flags: {}", analysis.flags.join(", ").dimmed());
                            }
                            
                            if analysis.score >= min_score {
                                quality_count += 1;
                            }
                            println!();
                        }
                        
                        if analysis.is_spam {
                            spam_count += 1;
                        }
                    }

                    println!("{}", "━".repeat(60).dimmed());
                    println!(
                        "📊 {} quality posts, {} filtered as spam",
                        quality_count.to_string().green(),
                        spam_count.to_string().red()
                    );
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
        }

        Commands::Analyze { title, content, author } => {
            let analysis = filter.analyze(&title, &content, author.as_deref());
            
            println!("\n{}", "📋 Spam Analysis".bold());
            println!("{}", "━".repeat(40));
            println!("Title: {}", title.cyan());
            println!("Score: {}/100", if analysis.score >= 50 { 
                analysis.score.to_string().green() 
            } else { 
                analysis.score.to_string().red() 
            });
            println!("Is Spam: {}", if analysis.is_spam { 
                "Yes".red() 
            } else { 
                "No".green() 
            });
            
            if !analysis.flags.is_empty() {
                println!("\nFlags:");
                for flag in &analysis.flags {
                    println!("  • {}", flag.yellow());
                }
            }
            
            if !analysis.positive_signals.is_empty() {
                println!("\nPositive signals:");
                for signal in &analysis.positive_signals {
                    println!("  ✓ {}", signal.green());
                }
            }
        }

        Commands::Rules => {
            println!("\n{}", "🔍 Spam Detection Rules".bold());
            println!("{}\n", "━".repeat(40));
            
            println!("{}", "❌ Negative Patterns (reduce score):".red());
            println!("  • CLAW/token minting spam (-40)");
            println!("  • Crypto shilling, token launches (-35)");
            println!("  • Prompt injection attempts (-50)");
            println!("  • Empty/minimal content (-30)");
            println!("  • Generic hourly check-ins (-25)");
            println!("  • Excessive emojis/buzzwords (-20)");
            println!("  • VC/promotional content (-30)");
            println!("  • Religious cult recruitment (-35)");
            println!("  • ALL CAPS shouting (-15)");
            
            println!("\n{}", "✓ Positive Signals (increase score):".green());
            println!("  • Technical content (+20)");
            println!("  • Code snippets (+15)");
            println!("  • Questions that invite discussion (+10)");
            println!("  • Reasonable length with substance (+10)");
            println!("  • References to other posts/agents (+5)");
            println!("  • Known quality authors (+15)");
        }
    }

    Ok(())
}
