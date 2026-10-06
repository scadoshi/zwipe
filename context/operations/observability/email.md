# Email checks

Two kinds of email matter: alerts that come to the owner, and transactional mail zerver sends to users.

## Alerts that arrive by email

| From | Means |
|---|---|
| UptimeRobot | `api.zwipe.net/health` is failing (it also pushes to the UptimeRobot app) |
| Healthchecks.io | The nightly zervice run never finished: no ping within the grace window |
| zervice-alert (via Resend) | A scheduled zervice run failed; the email says which |
| GitHub | A workflow on `main` failed |

Details in [`alerts.md`](alerts.md). Push alerts from the maintenance host go to ntfy, not email.

## Transactional email (Resend)

zerver sends verification and password-reset mail through Resend from the verified zwipe.net domain ([`../infrastructure/services.md`](../infrastructure/services.md)).

How to check it is working:

- **Is mail going out?** A failed send (`SendEmailError`, `zerver/src/lib/domain/email/`) is logged at error level as `email_send_failure` or `verification_email_failed`. Search the journal: `journalctl -u zerver --since today | grep -E 'email_send_failure|verification_email_failed'`.
- **Is it being delivered?** The Resend dashboard shows each send with its delivery, bounce and complaint status. A run of bounces or complaints is the early sign of a deliverability problem.
- **Is the domain still trusted?** SPF, DKIM and DMARC records live in Cloudflare ([`../infrastructure/cloudflare.md`](../infrastructure/cloudflare.md)); Resend's domain page shows whether they still verify.
- **Are users finishing verification?** `zcripts/metrics/funnel.sql` shows registrations against verified emails; a sudden drop in the verified share points at mail, not users.

Nothing alerts on transactional email today; these are read when something looks off or during a metrics pass.
