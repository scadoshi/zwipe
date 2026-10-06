# Alerts

Every channel that can tell the owner something is wrong, what it watches, and where it lands. Credentials, endpoints and addresses live on the machines that use them, never in this repo.

## Production (the VPS)

| Channel | Watches | Fires when | Lands as |
|---|---|---|---|
| UptimeRobot | `https://api.zwipe.net/health`, every 5 minutes | The endpoint fails or times out | Email and app push |
| Healthchecks.io dead-man check ("Zervice") | zervice pings it after a run where every step succeeded | No ping within the grace window after 04:00 UTC (zervice never ran or never finished) | Email |
| `zervice-alert.service` | `zervice.service` via `OnFailure=` | A scheduled zervice run exits non-zero | Email, sent through Resend with the server's existing credentials |
| Healthchecks.io dead-man check ("Zwipe Backups") | `backup-db.sh` pings it after the dump and the R2 upload both succeed | No ping within an hour after 05:00 UTC | Email |
| GitHub Actions | Deploy and test workflows on every push to `main` | A workflow fails | GitHub's own notification email |

`/health` is the probe on purpose: it is uncached and goes straight through to zerver, so it fails when zerver does, unlike cached routes that keep answering during an outage.

UptimeRobot and Healthchecks.io cover different failures: one says the API is down, the others say the nightly sync or the nightly backup didn't complete. `zervice-alert` covers zervice's third case: it ran and failed. The backup check was added 2026-10-06, after uploads had failed silently for 73 days (see [`../infrastructure/backups.md`](../infrastructure/backups.md)).

## Maintenance host (the owner's always-on box)

A set of cron jobs on the owner's home server watches the repos and pushes to a self-hosted [ntfy](https://ntfy.sh) server that is reachable only over the owner's private network (Tailscale). Access is deny-all by default: the jobs hold a write-only token and the owner's phone a read-only login. iOS delivery goes through ntfy.sh's push relay, which carries only an opaque poll request, never the message.

| Job | Schedule (UTC) | Pings when |
|---|---|---|
| repo-sync | hourly | A clean repo that is behind fails to fast-forward |
| ci-watch | every 30 minutes | A workflow's latest run fails. A run cancelled by a GitHub outage is rerun once, automatically, when GitHub reports Actions healthy |
| zervice-check | daily 05:15 | A local zervice run (against a local Postgres 18) shows retired or renamed oracle-tag slugs, a new tag without our description, or fails |
| ub-census | Mondays 06:00 | A new set appears that isn't in `universe.rs` FRANCHISES, a set outside FRANCHISES gains triangle-stamped printings, or a watched set first lists cards |
| reminders | daily 08:00 | A dated follow-up comes due |
| digest | Mondays 08:30 | Always: a weekly summary of repo state, the last zervice result, what's due in two weeks, and any job that has gone quiet |

The jobs alert; they don't commit. Fixes happen in a working session, following the runbooks (for tag churn, [`../../development/runbooks/otag_description_authoring.md`](../../development/runbooks/otag_description_authoring.md)).

## Gaps

- **In-app error and crash reports** are recorded server-side but alert nobody; they are read by hand on a schedule kept in [`../../progress/todo.md`](../../progress/todo.md).
