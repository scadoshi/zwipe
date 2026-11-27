# Local stack

Runs zwipe on this machine against a throwaway copy of the local dev database: migrated, seeded, zerver on :3100 and the web app on :8100.

```bash
zcripts/local/up.sh            # prints the URLs and the login when it's ready
zcripts/local/down.sh          # stops both and drops the copy
KEEP=1 zcripts/local/down.sh   # stops both and keeps the copy
```

Other ports: `ZERVER_PORT=3200 WEB_PORT=8200 zcripts/local/up.sh`, and the same for down.sh. Local databases only. The verify skill (`.claude/skills/verify/SKILL.md`) has the details and the checks it drives in Chrome.
