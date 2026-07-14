# Security Policy

## Supported versions

Zwipe ships as a live service: a hosted backend plus mobile and web clients. Only
the current release line is supported. Older client builds are expected to keep a
minimum version floor (`MIN_CLIENT_VERSION`) and may be asked to update.

| Component | Supported |
|-----------|-----------|
| Latest released app (iOS / Android / web) | Yes |
| Backend (`zerver`) — current deploy | Yes |
| Older client builds below the version floor | No |

## Reporting a vulnerability

Please report security issues privately. **Do not open a public GitHub issue for
a suspected vulnerability.**

Email **scottyfermo@hotmail.com** with:

- a description of the issue and its impact,
- steps to reproduce (or a proof of concept),
- affected component (`zerver`, `zwiper`, `zite`, `zwipe-core`) and version/build if known.

You can expect an acknowledgement within a few days. Once the issue is confirmed,
a fix will be prioritized and you'll be updated as it ships. Because the backend
is centrally hosted, most server-side fixes reach all users on the next deploy
without a client update.

## Scope

In scope: authentication and session handling, the REST API, data exposure
between users, and anything that lets one account act on another's behalf.

Out of scope: findings that require a rooted/jailbroken device or a compromised
build environment, rate-limiting or volumetric denial of service, and issues in
third-party services (Scryfall, Resend, the hosting provider) that we can only
pass upstream.

## Please avoid

- Accessing, modifying, or deleting data that isn't yours.
- Automated scanning that degrades the service for other users.
- Publicly disclosing the issue before a fix is available.

Good-faith research reported under this policy is welcome and appreciated.
