# Praxio - Your AI Assistant's AI Assistant

Save tokens, cut costs, and keep your AI focused on what matters. Praxio is a smart delegation layer for AI workflows that lets your main AI agent delegate specialized tasks to other models.

## Why Praxio?

Imagine you're working with Claude in your editor. You give it a complex task like "refactor this authentication module." Claude needs to understand the existing code, check for security issues, and plan the refactoring. 

**Without Praxio**: Claude burns through your token budget reading all the existing code into your shared conversation context, reducing how much space you have for actual work.

**With Praxio**: Claude asks a specialized model (like Gemini with its huge context window) to analyze the code in parallel, gets back a concise summary, and you keep your main conversation clean and focused.

### Real Problems It Solves

| Problem | Impact | Praxio Solution |
|---------|--------|-----------------|
| **Context Explosion** | Your 100k token limit fills up with background research | Delegate research tasks to separate models, keep main context clean |
| **Expensive For Simple Tasks** | Using Opus ($15/M tokens) to count lines of code | Route simple tasks to Haiku ($0.80/M) or Gemini Flash |
| **Can't Leverage Multiple Models** | Need Gemini's long context or Haiku's speed but stuck with one model | Use the right tool for each job in one workflow |
| **Slow Multi-Step Workflows** | Sequential tasks take forever | Run analysis in parallel on specialized models |
| **No Cost Tracking** | Don't know which tasks cost the most | See token usage and cost breakdown per delegation |

## Use Cases

### 1. **Code Analysis & Planning**
```
You: "Refactor the auth module to use OAuth2"
  ↓
Claude (your main agent):
  - Creates a refactoring plan
  - Delegates: "List all OAuth2 packages in package.json"
    to Haiku (cheap, fast)
  - Delegates: "Find all places auth tokens are used"  
    to Gemini (searches through large codebase)
  - Combines results into a detailed implementation plan
  - You stay focused on refactoring, not research
```

### 2. **Documentation Generation**
```
You: "Write API documentation for our REST endpoints"
  ↓
Claude:
  - Delegates: "Extract all route definitions and their current docs"
    to Gemini (better at pattern matching across files)
  - Delegates: "Generate OpenAPI spec from these routes"
    to Haiku (simpler task, costs $0.001)
  - Writes final, polished documentation
  - Total cost: ~$0.05 instead of $0.20 with one expensive model
```

### 3. **Security & Code Review**
```
You: "Review this codebase for security issues"
  ↓
Claude:
  - Delegates: "Find all database queries"
    to specialized model (SQL injection detection)
  - Delegates: "Find all HTTP endpoints"
    to another model (authentication checks)
  - Combines findings into comprehensive security report
  - Parallel execution = faster results
```

### 4. **Parallel Task Execution**
```
You: "Migrate this project from Express to Hono"
  ↓
Claude coordinates multiple delegations in parallel:
  - Analysis: "What Express patterns are used?" → Gemini
  - Planning: "How does Hono differ?" → Claude
  - Testing: "Generate migration tests" → Haiku
  - All three happen at once, results combined
```

## Getting Started

### Prerequisites

- Claude CLI installed and authenticated
- (Optional) Gemini CLI with API key for Google models
- Your text editor or Claude Code integration

### Installation

```bash
# Clone and build
git clone https://github.com/epistates/praxio.git
cd praxio
cargo build --release

# The binary is now at ./target/release/praxio
```

### Set Up with Claude Code

Add to your Claude Code configuration (usually `~/.config/claude/claude.json` or via UI):

```json
{
  "mcpServers": {
    "praxio": {
      "command": "/path/to/praxio/target/release/praxio"
    }
  }
}
```

Restart Claude Code, and you'll see two new tools available:
- **delegate_to_claude** - Use another Claude model
- **delegate_to_gemini** - Use a Google Gemini model

### Using Praxio

Once installed, just ask Claude naturally:

```
You: "I need to understand the codebase structure. 
      Can you ask a specialist to map out all the database tables and API endpoints?"

Claude will automatically use:
- invoke_claude to delegate "Map the database schema"
- invoke_gemini to delegate "Find all HTTP routes"

Then synthesize the results for you.
```

Or be explicit:

```
Claude, could you use Gemini to count lines of code across the project?
Claude will delegate to Gemini and return the results.
```

## Key Features

### Smart Delegation
- **Use the right model**: Route code analysis to Gemini's 1M context, simple tasks to speedy Haiku
- **Stay in context**: Main conversation stays focused on YOUR work
- **Parallel execution**: Multiple delegations happen simultaneously

### Cost Awareness
- **See what things cost**: Every delegation shows token usage and estimated cost
- **Choose models by efficiency**: Use cheaper models for simple tasks
- **Track total spending**: Know exactly how much you spent on delegation

### Reliable
- **Automatic fallback**: If your chosen model is busy, automatically try another
- **Session continuity**: Keep context across multiple delegations
- **Clear error messages**: Know exactly what went wrong and why

### Built for Integration
- **Works with Claude Code**: Seamless integration with your IDE
- **Works with any workflow**: Use with any MCP-compatible client
- **Extensible**: Future versions will support more models

## Examples

### Example 1: Quick Code Search

**What You'd Do Without Praxio:**
```
You (Claude Code): "Find all database queries in the codebase"
Claude reads entire codebase into context... 
Tokens used: 50,000
Cost: $0.25
```

**What You Do With Praxio:**
```
You: "Have Gemini search for all database queries"
Praxio delegates to Gemini (specialized search)
Tokens used: 3,000
Cost: $0.004
```

**Savings: 94% reduction in tokens, 98% cost reduction**

### Example 2: Code Review

**Without Praxio:**
```
Claude reviews security issues in main conversation
Context fills up with sensitive code examples
Total cost: $0.18 for one pass
```

**With Praxio:**
```
Claude delegates security check to Haiku
- XSS vulnerability detection: Haiku ($0.02)
- SQL injection patterns: Haiku ($0.02)
- Auth weaknesses: Haiku ($0.02)
Total cost: $0.06
```

**You save: $0.12 per security review, plus keep main context clean**

## Configuration

### Environment Variables

```bash
# Optional - Gemini support
export GEMINI_API_KEY="your-api-key"

# Optional - Debug logging
export RUST_LOG=info    # Show what's happening
export RUST_LOG=debug   # Very detailed logs
```

### Timeout Control

Each delegation supports custom timeouts:

```
"Can you delegate this to Claude with a 60-second timeout?"
```

Default timeouts:
- Claude: 30 seconds
- Gemini: 60 seconds (it's slower)

### Session Persistence

Keep context across delegations:

```
You: "Start a new analysis session"
Claude creates a session and returns a session_id

Later: "Continue the analysis session abc123"
Previous context is maintained
```

## Troubleshooting

### "Claude CLI not found"
Make sure Claude CLI is installed and in your PATH:
```bash
which claude  # Should show the path
claude --version  # Should show version number
```

### "Authentication failed"
Run `claude setup-token` and follow the authentication flow.

### "Gemini API key not found"
Set your Gemini API key:
```bash
export GEMINI_API_KEY="your-api-key"
```

### Delegation seems slow
- First delegation in a session takes ~2 seconds (startup time)
- Subsequent delegations are faster
- Gemini is slower than Claude by default (60s vs 30s timeout)

## How Much Does It Cost?

Praxio itself is **free and open source**. You only pay for the LLM API calls.

**Sample costs** (using public API pricing):

| Task | Traditional Approach | With Praxio | Savings |
|------|----------------------|-------------|---------|
| Search 100 files for patterns | Sonnet 20M tokens: $0.60 | Gemini 500K tokens: $0.03 | **95%** |
| Security code review | Opus 10M tokens: $0.75 | Haiku 1M tokens: $0.05 | **93%** |
| Documentation generation | Sonnet 15M tokens: $0.45 | Haiku 3M tokens: $0.02 | **96%** |

The more complex your project, the more you save.

## Limitations & Roadmap

### Current Limitations
- ✅ Claude provider fully supported
- ✅ Gemini provider fully supported  
- ❌ No OpenAI/Mistral support yet (coming in future)
- ❌ No response caching (each delegation re-runs)
- ❌ No automatic smart routing (you choose the model)

### Coming Soon
- **Phase 3**: Smart routing (Praxio suggests the best model automatically)
- **Phase 3**: Response caching (same query = instant answer)
- **Phase 4**: HTTP API for programmatic access
- **Phase 4**: More LLM providers (OpenAI, Mistral, local models)
- **Phase 4**: Cost reporting dashboard

## FAQ

**Q: Does Praxio send my code to extra services?**
A: Code only goes to Claude/Gemini APIs you explicitly choose. Praxio itself runs locally on your machine.

**Q: What if I don't have both Claude and Gemini?**
A: You only need Claude. Gemini support is optional - use Praxio with Claude alone if you prefer.

**Q: Can I use this without Claude Code?**
A: Yes, Praxio works with any MCP-compatible client (Cursor, continue.dev, etc).

**Q: Does Praxio collect analytics?**
A: No. Praxio is open source and completely local. No telemetry, no analytics, no tracking.

**Q: How is this different from just using one expensive model?**
A: Praxio lets Claude delegate intelligently - using cheaper/better-suited models for subtasks while Claude orchestrates. You get better results, faster, for less money.

**Q: Can I use this with local models?**
A: Not yet, but it's planned for Phase 4.

## Contributing

This project is open source under MIT license. We welcome contributions:

- **Found a bug?** Open an issue
- **Have an idea?** Start a discussion
- **Want to add a provider?** Fork and submit a PR
- **Documentation improvements?** PRs welcome!

## License

[MIT License](./LICENSE) - Use freely for any purpose

## Support

- **Issues & Bugs**: [GitHub Issues](https://github.com/epistates/praxio/issues)
- **Questions**: [GitHub Discussions](https://github.com/epistates/praxio/discussions)
- **Star the project** if you find it useful!
