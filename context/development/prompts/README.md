# Factory starting prompts

Paste one of these as the first message when a session starts or comes back from a compact. Each points at [`../factory.md`](../factory.md), which holds the rules. These prompts say which part of the rules is yours.

The pipeline:

1. **Scotty** asks for things, reports what he sees, makes product calls, and tests on his own phone and computer.
2. **Composer** logs every ask as a numbered issue, writes the brief, sends it to a worker, reviews and pushes, and sends QA to check. [`composer.md`](composer.md)
3. **Workers** (laptop, halo, thinker, scotland) trace the cause, fix it, prove it, and report. [`worker.md`](worker.md)
4. **QA** (surfer for the websites, droider for the Android app and phone web) shows the bug before, shows it gone after, and tries to break the fix. [`qa.md`](qa.md)

Then back to the composer, who closes it or sends it round again, and to Scotty, who has the last word.
