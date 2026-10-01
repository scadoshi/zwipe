# Release Notes Guidelines

A release has two pieces of outward text, both written from the version's block in `zwipe-core/src/content/changelog/mod.rs` after it moves to `RELEASES`. Neither says anything the changelog does not.

## The store text

`operations/store-submissions/<version>/whats_new.md`, one text pasted into both App Store Connect and the Play Console.

- Under Play's 500 characters, with a margin. Count with `wc -c` on the fenced block and write the count and the spare into the file's first line. The two consoles need not agree on how they count a newline, and a rejection at paste time is a bad place to find out.
- Same rules as a changelog entry: a phrase, no period at the end, no examples, no em dash. [`changelog_guidelines.md`](changelog_guidelines.md) is the rule; this file only adds the cap.
- Every changelog entry is represented, merged where two say one thing. Dropping one is allowed only when the cap forces it, and the file says which.
- Fixes and features in changelog order; the in-app changelog carries the full list, so the store text does not try to.
- In-app terms as the app uses them (command zone, Oracle tags, Profile), not paraphrased around.
- The file ends with one paragraph on what was compressed, dropped or renamed, and why.

## The Discord post

One message in the release channel, the day the build is submitted.

- First line: `**<version>** (<date>), in review on both stores`, or `live on both stores` when it is. Dates per [`versioning.md`](versioning.md).
- Then the changelog entries as bullets, verbatim from `RELEASES`, not the store compression.
- One closing line in Scott's voice. No roadmap, no apologies, no "stay tuned".
- Edited, not reposted, when the stores approve.

## What never goes in any of them

- A reason, a history or an apology: "we heard you", "after a bug in 1.10.2". Say what it does now.
- A promise about the next version.
- Internal names: build numbers, versionCodes, crate names, Scryfall, Cloudflare.
- Periods at the end of a bullet.

## Mechanics

Write the store text and the Discord post at cut time, before the builds start, from the moved changelog block. Record the submitted build in `operations/ios/app-store/submission/history.md` and `operations/android/play-store/submission/history.md` the moment it is uploaded, whether or not it ships; a number is burned on upload.
