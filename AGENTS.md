# terseid

Project type: library
Tools: `beads`, `maw`, `crit`, `botbus`, `botty`
Reviewer roles: correctness

---

<!-- Add project-specific context below: architecture, conventions, key files, etc. -->

<!-- botbox:managed-start -->
## Botbox Workflow

This project uses the botbox multi-agent workflow.

### Identity

Every command that touches bus or crit requires `--agent <name>`.
Use `<project>-dev` as your name (e.g., `terseid-dev`). Agents spawned by `agent-loop.sh` receive a random name automatically.
Run `bus whoami --agent $AGENT` to confirm your identity.

### Lifecycle

**New to the workflow?** Start with [worker-loop.md](.agents/botbox/worker-loop.md) — it covers the complete triage → start → work → finish cycle.

Individual workflow docs:

- [Close bead, merge workspace, release claims, sync](.agents/botbox/finish.md)
- [groom](.agents/botbox/groom.md)
- [Verify approval before merge](.agents/botbox/merge-check.md)
- [Validate toolchain health](.agents/botbox/preflight.md)
- [Report bugs/features to other projects](.agents/botbox/report-issue.md)
- [Reviewer agent loop](.agents/botbox/review-loop.md)
- [Request a review](.agents/botbox/review-request.md)
- [Handle reviewer feedback (fix/address/defer)](.agents/botbox/review-response.md)
- [Claim bead, create workspace, announce](.agents/botbox/start.md)
- [Find work from inbox and beads](.agents/botbox/triage.md)
- [Change bead status (open/in_progress/blocked/done)](.agents/botbox/update.md)
- [Full triage-work-finish lifecycle](.agents/botbox/worker-loop.md)

### Quick Start

```bash
AGENT=<project>-dev   # or: AGENT=$(bus generate-name)
bus whoami --agent $AGENT
br ready
```

### Beads Conventions

- Create a bead for each unit of work before starting.
- Update status as you progress: `open` → `in_progress` → `closed`.
- Reference bead IDs in all bus messages.
- Sync on session end: `br sync --flush-only`.

### Mesh Protocol

- Include `-L mesh` on bus messages.
- Claim bead: `bus claims stake --agent $AGENT "bead://$BOTBOX_PROJECT/<bead-id>" -m "<bead-id>"`.
- Claim workspace: `bus claims stake --agent $AGENT "workspace://$BOTBOX_PROJECT/$WS" -m "<bead-id>"`.
- Claim agents before spawning: `bus claims stake --agent $AGENT "agent://role" -m "<bead-id>"`.
- Release claims when done: `bus claims release --agent $AGENT --all`.

### Spawning Agents

1. Check if the role is online: `bus agents`.
2. Claim the agent lease: `bus claims stake --agent $AGENT "agent://role"`.
3. Spawn with an explicit identity (e.g., via botty or agent-loop.sh).
4. Announce with `-L spawn-ack`.

### Reviews

- Use `crit` to open and request reviews.
- If a reviewer is not online, claim `agent://reviewer-<role>` and spawn them.
- Reviewer agents loop until no pending reviews remain (see review-loop doc).

### Cross-Project Feedback

When you encounter issues with tools from other projects:

1. Query the `#projects` registry: `bus inbox --agent $AGENT --channels projects --all`
2. Find the project entry (format: `project:<name> repo:<path> lead:<agent> tools:<tool1>,<tool2>`)
3. Navigate to the repo, create beads with `br create`
4. Post to the project channel: `bus send <project> "Filed beads: <ids>. <summary> @<lead>" -L feedback`

See [report-issue.md](.agents/botbox/report-issue.md) for details.

### Stack Reference

| Tool | Purpose | Key commands |
|------|---------|-------------|
| bus | Communication, claims, presence | `send`, `inbox`, `claim`, `release`, `agents` |
| maw | Isolated jj workspaces | `ws create`, `ws merge`, `ws destroy` |
| br/bv | Work tracking + triage | `ready`, `create`, `close`, `--robot-next` |
| crit | Code review | `review`, `comment`, `lgtm`, `block` |
| botty | Agent runtime | `spawn`, `kill`, `tail`, `snapshot` |

### Loop Scripts

Scripts in `scripts/` automate agent loops:

| Script | Purpose |
|--------|---------|
| `agent-loop.sh` | Worker: sequential triage-start-work-finish |
| `dev-loop.sh` | Lead dev: triage, parallel dispatch, merge |
| `reviewer-loop.sh` | Reviewer: review loop until queue empty |
| `spawn-security-reviewer.sh` | Spawn a security reviewer |

Usage: `bash scripts/<script>.sh <project-name> [agent-name]`
<!-- botbox:managed-end -->
