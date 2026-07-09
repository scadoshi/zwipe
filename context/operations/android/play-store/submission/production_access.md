# Production access application (Play Console questionnaire)

New personal Play accounts must run closed testing (≥12 testers, 14 continuous
days) and then **apply for production access** via a questionnaire in the Play
Console (Test and release → Testing → Closed testing → *Apply for production*).
The answers below are pasted into that in-console form and reviewed by Google
(~72h). The 14-day cycle **completed 2026-07-09** (QA partner Teekam Suthar /
12testers); ~400 testers on the Alpha closed track.

## Framing rules (read before pasting)

- Say **"testers" / "testing program"**, never "users," "launch," or "release."
  Testers joined a closed test; the app was never on the production track.
- The 400 tester count is a **strength** (broad, active engagement), not a risk.
  Recruiting testers publicly via a public opt-in group is allowed and is *not*
  "treating it like live" (that would be distributing via production, or fake/
  install-only testers). We had real engagement, so say so.
- **Don't overclaim.** Frame it as a broad tester base plus a core of engaged
  testers (hired QA + active community) who filed detailed reports.
- Disclosing the paid testing provider is correct — Google's Q1 explicitly asks.

---

## ◈ About your closed test

**Q1. How did you recruit users for your closed test?**

> We recruited testers three ways: friends and coworkers, independent external
> testers engaged for unbiased feedback, and an open invitation to our
> closed-testing program shared on social media via a public opt-in group. This
> brought over 400 testers into the closed test, giving us a broad range of
> devices and real-world usage.

**Q2. How easy was it to recruit testers?** → **Easy**

**Q3. Describe the engagement you received from testers.**

> With 400+ testers opted in, we saw broad, active engagement across the core
> flows: account creation, swipe-based deck building, filtering, commander
> search, and deck import/export. A core group, including our external testers
> and active community members, went further and filed detailed bug reports and
> suggestions, while others exercised the app as typical Commander players would.

**Q4. Summary of the feedback received, and how you collected it.**

> Feedback covered UI/UX polish, stability, and ease of use, delivered as bug
> reports and concrete suggestions. We collected it through direct chat and group
> threads, which let us turn fixes around quickly and push updated builds during
> the test. Recurring themes were filter behavior, deck-management flow, and
> loading states, which we addressed across successive builds.

---

## ◈ About your app

**Q1. Who is the intended audience of your app?**

> Zwipe is built for Magic: The Gathering Commander (EDH) players who want a fast,
> mobile-first way to build, manage, and organize decks without the complexity of
> desktop deck builders.

**Q2. Describe how your app provides value to users.**

> Zwipe streamlines deck building with a swipe-based interface, powerful card
> filtering, deck statistics, import/export, and an automatically updated card
> database, making it faster to create and refine Commander decks on a phone.

**Q3. How many installs do you expect in your first year?** → **0–10k**

> Recommended for a niche first-year MTG app: more credible and less likely to
> invite scrutiny. (The QA partner suggested 10k–100k; either is defensible.)

---

## ◈ Your production readiness

**Q1. What changes did you make based on what you learned during the closed test?**

> Across the test we improved card filtering and search (sort-only and
> synergy-only searches now return results, and a Reset control returns each
> screen to its default view), added per-deck swipe memory so skipped and removed
> cards stay hidden, refined deck management with MVP card starring and shareable
> deck links, smoothed image and loading-skeleton states, and fixed several
> stability issues (session persistence on Android, a launch flash, and
> report-a-problem links now opening the OS mail app).

*(Reflects changes that actually shipped to testers during the cycle; excludes
back-swipe and per-screen filter persistence, which are in the not-yet-shipped
1.4.1 build.)*

**Q2. How did you decide that your app is ready for production?**

> We judged it production-ready after testing across a range of devices and
> Android versions through the closed-testing cycle. Tester feedback drove
> successive builds that resolved the reported bugs, and the final builds ran
> cleanly with no crashes and positive stability feedback, confirming it was
> ready to launch.
