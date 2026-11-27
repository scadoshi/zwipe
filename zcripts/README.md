# zcripts

Every script for working on zwipe that isn't part of an app. Each folder has a README with the details.

| Folder | What's in it |
|---|---|
| `local/` | Runs zerver and the web app on this machine against a seeded copy of the dev database |
| `dev-env/` | Sets up a fresh machine to run zwipe, and resets the local database |
| `seed/` | Fills a local database with test accounts, decks and cards |
| `web/` | zwiper-web's check before a commit |
| `ios/` | Simulator, device install and app icon helpers |
| `android/` | Patches for the Android project `dx bundle` generates, and a debug APK build |
| `server/` | systemd units and SQL installed on the production VPS |
| `metrics/` | Read-only usage reports to run against production |
| `latency/` | Quick API latency checks |
| `perf/` | Measures the deck page's Stacks at phone width |
| `wire-diff/` | Compares a candidate zerver's answers with main's before a merge |
| `synergy-worker/` | The Postgres role the synergy worker connects as |
| `deck-import/` | A prototype Archidekt importer |
| `themes/` | Renders every theme's colors to review a palette change |
