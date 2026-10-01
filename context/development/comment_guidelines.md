# Comment Guidelines

**A comment says what the code does, in the present tense, and only when the code does not say it itself.**

The code explains itself. A comment covers the part it cannot: the surprise, the invariant, the unit. A function with a comment on every line has the wrong names. Fix the names.

## Do not write

- **The history.** No "previously", "used to", "now", "no longer", "was changed to". No dates, commit hashes or issue numbers. The code has no past tense; git has the history.
- **The situation.** No "this is needed because we ran into", no "this prevents the bug where". If the code only makes sense with the story attached, the code is wrong, not the comment.
- **The sermon.** No "without this, X would happen, and then Y". Say what the line does. If the consequence is the part that is not obvious, give it one short clause.
- **A restatement.** `/// Returns the name.` above `fn name()` is noise. Delete it. When every item in a container explains itself, put `#[allow(missing_docs)]` on the container.
- **Advice.** Unless it is an instruction the reader must follow: "loopback only, do not widen".

## Do write

- What a caller can rely on: invariants, guarantees, ordering.
- The surprise: a reply that looks wrong but is right, an argument that is ignored, a close that looks early.
- Units, ranges and formats the type does not carry.

## Form

- One line when it can be. A doc comment may run longer, with an example when the signature cannot say it, and stops there.
- Every module opens with a one-line `//!` header.
- Every public item has a doc comment, unless it explains itself.
- Section banners are one line: `// == errors ==`.
- No em dashes, in code or in prose. No spaced hyphen joining two fragments either. Use a sentence, a colon, or a comma.
- American spelling.

## Before and after

```rust
// Bad: a past the reader does not need, and a sermon.
// Previously we kept the connection open after an error reply, but this caused
// a desync bug (see commit 3f2a9c1) where the next command read a stale error.
// Without this fix the cache would return wrong values.

// Good: what it does, and the one fact that is not obvious.
/// Puts `connection` back for the next command, unless the reply is an error.
///
/// A server that rejects a command partway through reading it answers the rest
/// as new commands, and those answers would be taken for the next replies.
```

```rust
// Bad: says nothing the field does not.
/// The additions.
pub additions: Option<u64>,

// Good: says what the number is and when it is absent.
/// Lines added across all commits, every rewrite counted again. `None` while
/// GitHub is still computing contributor statistics.
pub additions: Option<u64>,
```

## Checking

```bash
grep -rnE 'previously|used to|no longer|without this|in order to|—' src tests
```

A hit is not always wrong. "used to" also means "used for". Read each one. Then reread every comment you wrote and delete the ones a competent reader would not miss.
