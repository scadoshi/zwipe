# 499. Login, Register and Forgot password show each field's name twice

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Thinker's queue (composer, 2026-11-13): Login, Register and Forgot password show each label twice, as the label and again as the placeholder; drop the duplicate and keep the label, on the wide web, phone web and the app.

## Before
Headless, signed out, each field's label and placeholder: Login (Username or email, Password), Register (Username, Email, Password, Confirm password) and Forgot password (Email) repeat the label as the placeholder, 7 of 7 at 1440 and at 390. The app's three screens pass the same placeholders (`zwiper/src/lib/inbound/screens/auth/`).

## Cause
Each field passes `placeholder:` with its label's own text: `Field::placeholder()` in `zwipe_ui::screens::login` and `register` returns `label()`, and Forgot password passes `EMAIL` as both.

## Fix
scotland: the three screens on the web and in the app pass no placeholder, and `placeholder()` goes from both `Field`s. The account forms' placeholders are left as they are.

## Verification
- Tests in `zwiper-web` and `zwiper` `screens/auth/mod.rs`: no auth screen passes a placeholder (red before, in both crates).
- Headless at 1440 and 390: the 7 fields show their labels with no placeholder.
