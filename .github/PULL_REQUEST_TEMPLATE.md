<!--
Thanks for the change. Keep the summary tight and check the boxes that apply.
Note: Zwipe is licensed noncommercially (see LICENSE) and is primarily maintained
by its owner. Large unsolicited changes may not be merged, so open an issue to
discuss anything substantial first.
-->

## Summary

<!-- What does this change do, and why? One or two sentences. -->

## Checklist

- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean
- [ ] `cargo +nightly fmt` has been run (CI's fmt gate is nightly)
- [ ] If any SQL query changed: ran `cargo sqlx prepare --workspace` and committed `.sqlx/`
- [ ] Domain changes respect the `zwipe-core` purity rules (no server-only deps, no SQLx derives)
- [ ] Commit messages are concise, no emojis, no AI-agent signatures

## Notes

<!-- Anything reviewers should know: follow-ups, trade-offs, screenshots. -->
