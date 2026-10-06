# Observability

How we know zwipe is running well: every signal, where it comes from, and where to read it.

## The routes

| Signal | Source | Read it in | Alerts? |
|---|---|---|---|
| API up | `GET /health` (server and database), `/health/server`, `/health/database`; `GET /` returns the running version | UptimeRobot, or curl it | Yes: UptimeRobot email and push |
| Nightly sync ran | zervice pings Healthchecks.io after a fully successful run | Healthchecks.io | Yes: email when the ping is missed |
| Nightly sync failed | `OnFailure=` on `zervice.service` | Email from zervice-alert | Yes: email |
| Deploys and tests | GitHub Actions on `main` | GitHub, `gh run list` | Yes: GitHub email; the maintenance host's ci-watch pushes too |
| Repo, tag and set upkeep | Cron jobs on the maintenance host | ntfy on the owner's phone | Yes: push, plus a Monday digest |
| Server logs | zerver and zervice log to the journal and `LOG_DIR` (default `/var/log/zwipe`) | `journalctl -u zerver`, `journalctl -u zervice` | No |
| In-app errors and crashes | Apps report handled errors and panics to `client_errors` and `crash_reports` (90-day retention) | `zcripts/metrics/errors.sql` | No: read on a schedule in [`../../progress/todo.md`](../../progress/todo.md) |
| Usage | Counters and events in the database | `zcripts/metrics/` scripts ([`README`](../../../zcripts/metrics/README.md)) | No |
| Email | Resend sends; alerts arrive by email | [`email.md`](email.md) | Partly |
| Backups | Nightly `pg_dump` to object storage | [`../infrastructure/backups.md`](../infrastructure/backups.md) | No (a known gap) |

## In this folder

- [`alerts.md`](alerts.md): every alert channel in detail, the maintenance host's jobs, and the known gaps.
- [`email.md`](email.md): alerts that arrive by email, and how to check transactional mail.
