# Server

Files installed on the production VPS. Nothing here runs from a checkout.

- `systemd/` holds the units: `zerver.service` (the API), `zervice.service` and `zervice.timer` (the nightly Scryfall sync, 04:00 UTC), and `zervice-alert.service` with `zervice-alert.sh`, which emails when a nightly run fails.
- `sql/zervice_role.sql` grants the `zervice` role what the sync needs, including ownership of the materialized views it refreshes. Run it again after a migration recreates one of them.

How they're installed is in `zwipe-context operations/infrastructure/server.md`.
