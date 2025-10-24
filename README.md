# Praxio - Your AI Assistant's AI Assistant

Save tokens, cut costs, and keep your AI focused on what matters. Praxio is a smart delegation layer for AI workflows that lets any AI agent delegate specialized tasks to other models—using Claude, Gemini, or any combination.

## Why Praxio?

Imagine you're working with an AI in your editor (Claude, Gemini, or any MCP-compatible agent). You give it a complex task like "refactor this authentication module." Your AI needs to understand the existing code, check for security issues, and plan the refactoring.

**Without Praxio**: Your AI burns through your token budget reading all the existing code into your shared conversation context, reducing how much space you have for actual work.

**With Praxio**: Your AI delegates specialized tasks to other models—using Claude's speed, Gemini's huge context window, or both—gets back concise summaries, and you keep your main conversation clean and focused on what matters.

### What It Solves

- **Context Window Pollution**: Keep your main AI conversation focused by delegating background tasks instead of polluting it with large file reads
- **Expensive For Simple Tasks**: Route simple subtasks to less expensive models while your main AI stays focused
- **Leverage Multiple Models**: Use Claude for reasoning, Gemini for large-scale analysis, or any combination in one workflow
- **Parallel Task Execution**: Run multiple delegations simultaneously instead of sequentially
- **Token Efficiency**: See exactly which tasks are consuming tokens and delegate accordingly

## Multi-Provider Support

Praxio works with multiple LLM providers:

- **Claude** - Fast, excellent reasoning, good for coordination
- **Gemini** - Huge context window (1M+ tokens), great for large file analysis
- **Easy to combine** - Use Claude for logic, Gemini for analysis, in the same workflow

Choose the best model for each task, not just one model for everything.

## Use Cases

Your AI agent can intelligently delegate:

- **Code Analysis**: Analyze large codebases without filling main context
- **Documentation Generation**: Generate docs from pattern matching across files
- **Security & Code Review**: Run multiple security checks in parallel
- **Codebase Navigation**: Search and map files using specialized models
- **Multi-Step Tasks**: Orchestrate complex workflows with different models

## Getting Started

### Prerequisites

- **One or both LLM CLIs installed and authenticated:**
  - Claude CLI installed and authenticated, OR
  - Gemini CLI with API key set, OR
  - Both for maximum flexibility
- Your text editor or MCP-compatible client integration

### Installation

```bash
# Clone and build
git clone https://github.com/epistates/praxio.git
cd praxio
cargo build --release

# The binary is now at ./target/release/praxio
```

### Set Up with Claude Code (or any MCP client)

Add to your configuration (usually `~/.config/claude/claude.json` or via UI):

```json
{
  "mcpServers": {
    "praxio": {
      "command": "/path/to/praxio/target/release/praxio"
    }
  }
}
```

Restart your client, and you'll see tools available:
- **invoke_claude** - Delegate to Claude models
- **invoke_gemini** - Delegate to Google Gemini models

### Using Praxio

Once installed, you can ask your AI agent naturally:

```
You: "I need to understand the codebase structure. 
      Can you delegate analyzing the database schema and API endpoints?"

Your AI will automatically use delegation to handle this in parallel
and synthesize the results for you.
```

Or be explicit about which provider to use:

```
"Can you use Gemini to analyze all files in this directory? 
It has a huge context window."

"Use Claude to quickly review this code for bugs."
```

## Key Features

### Smart Delegation Across Providers
- **Claude**: Fast, excellent for reasoning and coordination
- **Gemini**: 1M context window, great for large-scale analysis
- **Right tool for the job**: Use each model's strengths
- **Stay in context**: Main conversation stays focused on YOUR work
- **Parallel execution**: Multiple delegations happen simultaneously

### Cost Optimization
- **Delegate smartly**: Use less expensive models for simpler tasks
- **Choose by efficiency**: Route different tasks to different models
- **See token usage**: Track which delegations use most tokens
- **Reduce context costs**: Smaller main context = lower token usage

### Reliable
- **Automatic fallback** (Claude): If your chosen model is busy, automatically try another
- **Session continuity**: Keep context across multiple delegations
- **Clear error messages**: Know exactly what went wrong and why

### Built for Any Workflow
- **Works with Claude Code**: Seamless integration
- **Works with any MCP client**: Cursor, continue.dev, etc
- **Provider agnostic**: Add Claude, Gemini, or both
- **Extensible**: Future support for more models

## Configuration

### Environment Variables

```bash
# Optional - Claude support
export CLAUDE_CLI_PATH="/path/to/claude"  # Usually auto-detected

# Optional - Gemini support
export GEMINI_API_KEY="your-api-key"

# Optional - Debug logging
export RUST_LOG=info    # Show what's happening
export RUST_LOG=debug   # Very detailed logs
```

### Provider Timeouts

Each provider has sensible defaults, configurable per delegation:

```
"Delegate this to Claude with a 60-second timeout"
"Use Gemini with 90 seconds since it searches large contexts"
```

Default timeouts:
- Claude: 30 seconds (fast responses)
- Gemini: 60 seconds (larger contexts take time)

### Session Persistence

Keep context across delegations:

```
You: "Start a new analysis session"
Your agent creates a session and returns a session_id

Later: "Continue the analysis session abc123"
Previous context is maintained across providers
```

## Troubleshooting

### "Claude CLI not found"
Make sure Claude CLI is installed and in your PATH:
```bash
which claude  # Should show the path
claude --version  # Should show version number
```

### "Gemini API key not found"
Set your Gemini API key:
```bash
export GEMINI_API_KEY="your-api-key"
```

### "Authentication failed for Claude"
Run `claude setup-token` and follow the authentication flow.

### Delegation seems slow
- First delegation in a session takes ~2 seconds (startup time)
- Subsequent delegations are faster
- Claude: Generally ~1-3 seconds per delegation
- Gemini: Generally ~3-8 seconds (analyzing larger contexts)

### Provider availability errors
Praxio checks provider availability on startup:
- Claude: Checks if CLI exists and responds
- Gemini: Checks for GEMINI_API_KEY environment variable
- Both optional: Use whichever you have configured

## Supported Providers

### Currently Supported
- ✅ **Claude** - All Claude models via Claude CLI
- ✅ **Gemini** - All Gemini models via Gemini CLI

### Coming Soon
- 🚧 **OpenAI** - GPT models
- 🚧 **Mistral** - Mistral models
- 🚧 **Local models** - Ollama, LM Studio
- 🚧 **Azure** - Claude and others via Azure

## Limitations & Roadmap

### Current Capabilities
- ✅ Multi-provider support (Claude + Gemini)
- ✅ Session management with context continuity
- ✅ Token and cost tracking
- ✅ Parallel delegation execution
- ✅ Automatic fallback (Claude)

### Coming Soon - Phase 3
- 🚧 Smart routing (Praxio suggests best model automatically)
- 🚧 Response caching (same query = instant answer)
- 🚧 Provider composition (combine multiple models per task)
- 🚧 Extended thinking mode (Gemini's reasoning tokens)

### Future - Phase 4
- 🚧 HTTP API for programmatic access
- 🚧 More LLM providers (OpenAI, Mistral, local)
- 🚧 Delegation metrics and insights
- 🚧 Model performance tracking

## FAQ

**Q: Which provider should I use?**
A: Both! Claude is fast and great for coordination. Gemini has 1M context perfect for large file analysis. Use them together for best results.

**Q: Does Praxio send my code to extra services?**
A: Code only goes to the providers (Claude/Gemini APIs) you explicitly choose. Praxio itself runs locally on your machine.

**Q: What if I only have Claude?**
A: That's fine. Gemini support is optional - use Praxio with just Claude. Later, add Gemini when you need its 1M context.

**Q: What if I only have Gemini?**
A: Also fine. Use just Gemini's huge context window for large-scale analysis.

**Q: Can I use this without Claude Code?**
A: Yes, Praxio works with any MCP-compatible client (Cursor, continue.dev, etc).

**Q: Does Praxio collect analytics?**
A: No. Praxio is open source and completely local. No telemetry, no analytics, no tracking.

**Q: How does delegation help my workflow?**
A: Delegation keeps your main AI focused on your task, not drowning in background research. You get faster responses, cleaner context, and can choose the best tool for each job.

**Q: Can I use this with local models?**
A: Not yet, but it's planned for Phase 4.

**Q: Can I add my own provider?**
A: Yes! Praxio is extensible. See Contributing section.

## Contributing

This project is open source under MIT license. We welcome contributions:

- **Found a bug?** Open an issue
- **Have an idea?** Start a discussion
- **Want to add a provider?** Fork and submit a PR - Praxio uses trait-based providers
- **Documentation improvements?** PRs welcome!

## License

[MIT License](./LICENSE) - Use freely for any purpose

## Support

- **Issues & Bugs**: [GitHub Issues](https://github.com/epistates/praxio/issues)
- **Questions**: [GitHub Discussions](https://github.com/epistates/praxio/discussions)
- **Star the project** if you find it useful!
