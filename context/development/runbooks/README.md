# Runbooks

Repeatable, AI-orchestrated procedures a fresh assistant can run cold. Each runbook
is a step-by-step loop (often fanning out subagents via the `Workflow` tool) with the
exact queries, scripts, validation, and gotchas needed to reproduce a result.

| Runbook | What it does |
|---------|--------------|
| [`otag_description_authoring.md`](otag_description_authoring.md) | Author `ORACLE_TAG_DESCRIPTIONS` in batches, draft + oracle-text-verify, splice into the const. Ships [`otag_authoring_workflow.js`](otag_authoring_workflow.js). |
| [`otag_description_audit.md`](otag_description_audit.md) | Second-pass QA: audit + independently verify authored descriptions against real cards, flag inaccuracies (findings only). Ships [`otag_audit_workflow.js`](otag_audit_workflow.js). Includes the false-flag incident writeup and a regression-watch checklist. |

**Opt-in:** runbooks that fan out agents use `Workflow`, which the user must
explicitly authorize. Don't launch unprompted.
