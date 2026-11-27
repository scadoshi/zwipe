# zcripts

Every script for working on zwipe that isn't part of an app. Each folder has a README with the details.

| Folder | What's in it |
|---|---|
| `development/` | Sets up a machine, seeds a local database, and runs zerver and the web app against a seeded copy |
| `web/` | zwiper-web's check before a commit |
| `ios/` | Simulator, device install and app icon helpers |
| `android/` | Patches for the Android project `dx bundle` generates, and a debug APK build |
| `server/` | systemd units installed on the production VPS, and the database roles (the synergy worker's is a prototype) |
| `metrics/` | Read-only usage reports to run against production |
| `performance/` | API latency checks and the deck page's Stacks probe |
| `wire-diff/` | Compares a candidate zerver's answers with main's before a merge |
| `deck-import/` | A prototype Archidekt importer |
| `themes/` | Renders every theme's colors to review a palette change |
