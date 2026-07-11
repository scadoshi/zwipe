# Upload to the Play Console + roll out

Takes the signed `.aab` from [build.md](build.md) and gets it onto a testing
track. Listing copy is in [form_fields.md](form_fields.md).

---

## 1. Upload to the Play Console

Listing + content forms must be complete (no errors) first — there is **no
separate "submit listing for review"** step; rolling out a release reviews app +
listing + content together.

1. **Test and release → Testing → Closed testing → Create new release**
   (new personal accounts must run **Closed testing**: ≥12 testers, 14 continuous
   days, before Production access. Internal testing is fine for pipeline checks
   but does **not** count toward the 14 days.)
2. First release: enroll in **Play App Signing → "Use Google-generated key"**
   (Google holds the app signing key; `zwipe-upload.jks` is just the upload key).
3. **Upload** `zwipe-<VERSION>.aab`.
4. **Release name** `<versionName> (<versionCode>)`, e.g. `1.0.9 (1)` (internal only).
   **Release notes** inside `<en-US>…</en-US>` (≤500 chars, generic "TCG" wording).
5. **Save → Review → Start rollout to closed testing.**
6. **Testers** tab → add testers, then share the opt-in link. The 14-day clock
   runs from when ≥12 are opted in.

**Promoting to Production (after the 14-day gate):** create the release under
**Test and release → Production**. The Production track starts with **no countries
selected** — the "no countries or regions" error blocks the release until you set
them at **Test and release → Production → Countries/regions → Add / Select all**.
This is per-track (closed testing's country list does *not* carry over) and lives on
the track, **not** the release page or the bundle. (First hit 2026-07-11.)

## 2. Recruiting testers (the gotchas)

- Closed testing is **invite-only** — only emails on your tester list (or members
  of an added **Google Group**) can join. A bare public link does nothing for
  someone not on the list.
- **For social-media recruitment, use a Google Group**, not a hand-typed email
  list: create a public group at groups.google.com ("Anyone can join"), then
  Testers → **Google Groups** → paste the group address → Save. Anyone who joins
  the group becomes an eligible tester automatically — no per-person adding. Post
  the group join link + the opt-in URL together.
- The **"Copy link" stays greyed until the release clears review and is live** on
  the track ("the link will be shown when you publish"). The opt-in URL is
  predictable, though: `https://play.google.com/apps/testing/com.scadoshi.zwipe`
  — it only works post-publish and only for list/group members.
- Also fill the **Feedback URL or email** field so testers know where to report.

---

## Native debug symbols (the "upload a symbol file" warning)

Play shows a **non-blocking warning** that the bundle has native code without
debug symbols. **Ship anyway — it's cosmetic.** AGP's `ndk { debugSymbolLevel }`
only harvests symbols from libraries *it* builds (CMake/ndk-build); it does
**not** touch a **prebuilt** `.so`, and dx drops the Rust lib straight into
`jniLibs/`. So that config is a no-op here and the warning persists regardless.

The `.so` itself is unstripped (`file …/jniLibs/arm64-v8a/libmain.so` →
"not stripped, with debug_info"), so *if* you ever need symbolicated native
crash reports, upload symbols **manually**: Play Console → App bundle explorer →
the bundle → **Upload native debug symbols** (a zip containing
`arm64-v8a/libmain.so`). Not worth it for routine releases.
