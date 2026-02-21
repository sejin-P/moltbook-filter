use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, Read};

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

    // === INTERACTION COMMANDS ===

    /// Create a new post on Moltbook
    Post {
        /// Moltbook API key
        #[arg(short, long, env = "MOLTBOOK_API_KEY")]
        api_key: String,

        /// Post title
        #[arg(short, long)]
        title: String,

        /// Post content (use - to read from stdin)
        #[arg(short, long)]
        content: String,

        /// Submolt to post in (e.g., "philosophy", "tech")
        #[arg(short = 'm', long)]
        submolt: Option<String>,
    },
    /// Upvote a post
    Upvote {
        /// Moltbook API key
        #[arg(short, long, env = "MOLTBOOK_API_KEY")]
        api_key: String,

        /// Post ID to upvote
        #[arg(short, long)]
        post_id: String,
    },
    /// Downvote a post
    Downvote {
        /// Moltbook API key
        #[arg(short, long, env = "MOLTBOOK_API_KEY")]
        api_key: String,

        /// Post ID to downvote
        #[arg(short, long)]
        post_id: String,
    },
    /// Remove vote from a post
    Unvote {
        /// Moltbook API key
        #[arg(short, long, env = "MOLTBOOK_API_KEY")]
        api_key: String,

        /// Post ID to unvote
        #[arg(short, long)]
        post_id: String,
    },
    /// Add a comment to a post
    Comment {
        /// Moltbook API key
        #[arg(short, long, env = "MOLTBOOK_API_KEY")]
        api_key: String,

        /// Post ID to comment on
        #[arg(short, long)]
        post_id: String,

        /// Comment content (use - to read from stdin)
        #[arg(short = 'm', long)]
        message: String,
    },
    /// View comments on a post
    Comments {
        /// Moltbook API key
        #[arg(short, long, env = "MOLTBOOK_API_KEY")]
        api_key: String,

        /// Post ID to view comments for
        #[arg(short, long)]
        post_id: String,
    },
    /// View your profile stats
    Profile {
        /// Moltbook API key
        #[arg(short, long, env = "MOLTBOOK_API_KEY")]
        api_key: String,

        /// Username to lookup (defaults to your own profile)
        #[arg(short, long)]
        user: Option<String>,
    },
    /// View a specific post by ID
    View {
        /// Moltbook API key
        #[arg(short, long, env = "MOLTBOOK_API_KEY")]
        api_key: String,

        /// Post ID to view
        #[arg(short, long)]
        post_id: String,
    },
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
                            println!("    by {} in m/{} [id:{}]", 
                                post.author.as_deref().unwrap_or("unknown").cyan(),
                                post.submolt.as_deref().unwrap_or("?"),
                                post.id.dimmed()
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

        // === INTERACTION COMMANDS ===

        Commands::Post { api_key, title, content, submolt } => {
            let client = MoltbookClient::new(api_key);
            
            // Support reading content from stdin
            let actual_content = if content == "-" {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf).expect("Failed to read stdin");
                buf.trim().to_string()
            } else {
                content
            };

            println!("{}", "📝 Creating post...".cyan());

            match client.create_post(&title, &actual_content, submolt.as_deref()).await {
                Ok(post) => {
                    println!("\n{}", "✓ Post created!".green().bold());
                    println!("{}", "━".repeat(40));
                    println!("Title: {}", post.title.bold());
                    println!("ID: {}", post.id.cyan());
                    if let Some(sub) = &post.submolt {
                        println!("Submolt: m/{}", sub);
                    }
                    println!("URL: https://www.moltbook.com/post/{}", post.id);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
        }

        Commands::Upvote { api_key, post_id } => {
            let client = MoltbookClient::new(api_key);
            println!("{}", "👍 Upvoting...".cyan());

            match client.upvote(&post_id).await {
                Ok(()) => {
                    println!("{} Post {} upvoted!", "✓".green(), post_id);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
        }

        Commands::Downvote { api_key, post_id } => {
            let client = MoltbookClient::new(api_key);
            println!("{}", "👎 Downvoting...".cyan());

            match client.downvote(&post_id).await {
                Ok(()) => {
                    println!("{} Post {} downvoted!", "✓".green(), post_id);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
        }

        Commands::Unvote { api_key, post_id } => {
            let client = MoltbookClient::new(api_key);
            println!("{}", "↩ Removing vote...".cyan());

            match client.unvote(&post_id).await {
                Ok(()) => {
                    println!("{} Vote removed from post {}!", "✓".green(), post_id);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
        }

        Commands::Comment { api_key, post_id, message } => {
            let client = MoltbookClient::new(api_key);
            
            // Support reading from stdin
            let actual_message = if message == "-" {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf).expect("Failed to read stdin");
                buf.trim().to_string()
            } else {
                message
            };

            println!("{}", "💬 Adding comment...".cyan());

            match client.comment(&post_id, &actual_message).await {
                Ok(comment) => {
                    println!("\n{}", "✓ Comment added!".green().bold());
                    println!("{}", "━".repeat(40));
                    println!("ID: {}", comment.id.cyan());
                    println!("Content: {}", comment.content);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
        }

        Commands::Comments { api_key, post_id } => {
            let client = MoltbookClient::new(api_key);
            println!("{}", "💬 Fetching comments...".cyan());

            match client.get_comments(&post_id).await {
                Ok(comments) => {
                    if comments.is_empty() {
                        println!("\nNo comments yet.");
                    } else {
                        println!("\n{} comments:\n", comments.len());
                        for comment in comments {
                            println!("{}", "━".repeat(40).dimmed());
                            println!(
                                "{} • {} upvotes",
                                comment.author.as_deref().unwrap_or("anon").cyan(),
                                comment.upvotes
                            );
                            println!("{}", comment.content);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
        }

        Commands::Profile { api_key, user } => {
            let client = MoltbookClient::new(api_key);
            println!("{}", "👤 Fetching profile...".cyan());

            let result = match user {
                Some(username) => client.get_profile(&username).await,
                None => client.get_my_profile().await,
            };

            match result {
                Ok(profile) => {
                    println!("\n{}", "━".repeat(40));
                    println!("👤 {}", profile.name.bold());
                    println!("{}", "━".repeat(40));
                    println!("   Karma: {}", profile.karma.to_string().green());
                    println!("   Followers: {}", profile.followers.to_string().cyan());
                    println!("   Following: {}", profile.following);
                    println!("   Posts: {}", profile.post_count);
                    println!("   Comments: {}", profile.comment_count);
                    if let Some(bio) = &profile.bio {
                        println!("\n   Bio: {}", bio.dimmed());
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
        }

        Commands::View { api_key, post_id } => {
            let client = MoltbookClient::new(api_key);
            println!("{}", "📖 Fetching post...".cyan());

            match client.get_post(&post_id).await {
                Ok(post) => {
                    let analysis = filter.analyze(&post.title, &post.content, post.author.as_deref());
                    
                    println!("\n{}", "━".repeat(60));
                    println!("{}", post.title.bold());
                    println!(
                        "by {} in m/{} • {} upvotes • {} comments",
                        post.author.as_deref().unwrap_or("unknown").cyan(),
                        post.submolt.as_deref().unwrap_or("?"),
                        post.upvotes,
                        post.comment_count
                    );
                    println!("{}", "━".repeat(60));
                    println!("\n{}\n", post.content);
                    println!("{}", "━".repeat(60));
                    println!(
                        "Quality score: {}/100 {}",
                        if analysis.score >= 50 { analysis.score.to_string().green() } 
                        else { analysis.score.to_string().red() },
                        if analysis.is_spam { "(spam)".red() } else { "".normal() }
                    );
                    println!("URL: https://www.moltbook.com/post/{}", post.id);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
        }
    }

    Ok(())
}
