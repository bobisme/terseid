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

Every command that touches botbus or crit requires `--agent <name>`.
Use `<project>-dev` as your name (e.g., `terseid-dev`). Agents spawned by `agent-loop.sh` receive a random name automatically.
Run `botbus whoami --agent $AGENT` to confirm your identity.

### Lifecycle

**New to the workflow?** Start with [worker-loop.md](.agents/botbox/worker-loop.md) — it covers the complete triage → start → work → finish cycle.

Individual workflow docs:

- [Close bead, merge workspace, release claims, sync](.agents/botbox/finish.md)
- [Verify approval before merge](.agents/botbox/merge-check.md)
- [Validate toolchain health](.agents/botbox/preflight.md)
- [Report bugs/features to other projects](.agents/botbox/report-issue.md)
- [Reviewer agent loop](.agents/botbox/review-loop.md)
- [Request a review](.agents/botbox/review-request.md)
- [Claim bead, create workspace, announce](.agents/botbox/start.md)
- [Find work from inbox and beads](.agents/botbox/triage.md)
- [Change bead status (open/in_progress/blocked/done)](.agents/botbox/update.md)
- [Full triage-work-finish lifecycle](.agents/botbox/worker-loop.md)

### Quick Start

```bash
AGENT=<project>-dev   # or: AGENT=$(botbus generate-name)
botbus whoami --agent $AGENT
br ready
```

### Beads Conventions

- Create a bead for each unit of work before starting.
- Update status as you progress: `open` → `in_progress` → `closed`.
- Reference bead IDs in all botbus messages.
- Sync on session end: `br sync --flush-only`.

### Mesh Protocol

- Include `-L mesh` on botbus messages.
- Claim bead: `botbus claim --agent $AGENT "bead://$BOTBOX_PROJECT/<bead-id>" -m "<bead-id>"`.
- Claim workspace: `botbus claim --agent $AGENT "workspace://$BOTBOX_PROJECT/$WS" -m "<bead-id>"`.
- Claim agents before spawning: `botbus claim --agent $AGENT "agent://role" -m "<bead-id>"`.
- Release claims when done: `botbus release --agent $AGENT --all`.

### Spawning Agents

1. Check if the role is online: `botbus agents`.
2. Claim the agent lease: `botbus claim --agent $AGENT "agent://role"`.
3. Spawn with an explicit identity (e.g., via botty or agent-loop.sh).
4. Announce with `-L spawn-ack`.

### Reviews

- Use `crit` to open and request reviews.
- If a reviewer is not online, claim `agent://reviewer-<role>` and spawn them.
- Reviewer agents loop until no pending reviews remain (see review-loop doc).

### Cross-Project Feedback

When you encounter issues with tools from other projects:

1. Query the `#projects` registry: `botbus inbox --agent $AGENT --channels projects --all`
2. Find the project entry (format: `project:<name> repo:<path> lead:<agent> tools:<tool1>,<tool2>`)
3. Navigate to the repo, create beads with `br create`
4. Post to the project channel: `botbus send <project> "Filed beads: <ids>. <summary> @<lead>" -L feedback`

See [report-issue.md](.agents/botbox/report-issue.md) for details.

### Stack Reference

| Tool   | Purpose                         | Key commands                                  |
| ------ | ------------------------------- | --------------------------------------------- |
| botbus | Communication, claims, presence | `send`, `inbox`, `claim`, `release`, `agents` |
| maw    | Isolated jj workspaces          | `ws create`, `ws merge`, `ws destroy`         |
| br/bv  | Work tracking + triage          | `ready`, `create`, `close`, `--robot-next`    |
| crit   | Code review                     | `review`, `comment`, `lgtm`, `block`          |
| botty  | Agent runtime                   | `spawn`, `kill`, `tail`, `snapshot`           |

<!-- botbox:managed-end -->
