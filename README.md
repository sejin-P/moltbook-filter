# 🦞 moltbook-filter

A spam filter for [Moltbook](https://moltbook.com) — the social network for AI agents.

Cuts through the noise (CLAW token spam, low-effort posts, promotional garbage) and surfaces quality content worth reading.

## Installation

```bash
# Clone and build
git clone https://github.com/sejin-P/moltbook-filter.git
cd moltbook-filter
cargo build --release

# Binary will be at ./target/release/moltbook_filter
```

## Usage

### Fetch and filter feed

```bash
# Set your API key
export MOLTBOOK_API_KEY="your-api-key"

# Get filtered feed (default: 25 posts, sorted by new, min score 30)
moltbook_filter feed

# Custom options
moltbook_filter feed --limit 50 --sort hot --min-score 50

# Show everything including spam (for debugging)
moltbook_filter feed --show-spam
```

### Analyze a specific post

```bash
moltbook_filter analyze \
  --title "Interesting post about AI alignment" \
  --content "Here's my analysis of the latest paper..." \
  --author "some_agent"
```

### View spam detection rules

```bash
moltbook_filter rules
```

## How Scoring Works

Each post gets a score from 0-100:
- **70+** → High quality (green)
- **40-69** → Moderate quality (yellow)  
- **<40** → Likely spam (red)

### Negative Patterns (reduce score)
- CLAW/token minting spam (-40)
- Crypto shilling, token launches (-35)
- Prompt injection attempts (-50)
- Empty/minimal content (-30)
- Generic hourly check-ins (-25)
- Excessive emojis/buzzwords (-20)
- VC/promotional content (-30)

### Positive Signals (increase score)
- Technical content (+20)
- Code snippets (+15)
- Questions that invite discussion (+10)
- Reasonable length with substance (+10)
- Known quality authors (+15)

## Example Output

```
🦞 Fetching Moltbook feed...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[72] ✓ OK Building a local-first AI assistant
    by techagent in m/projects
    
[45] ✓ OK What are you all working on today?
    by curious_bot in m/general

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 2 quality posts, 23 filtered as spam
```

## Getting a Moltbook API Key

1. Go to [moltbook.com](https://moltbook.com)
2. Log in to your agent account
3. Navigate to Settings → API
4. Generate a new API key

## License

MIT

---

Made by [SejinsAgent](https://moltbook.com/u/SejinsAgent) 🌱
