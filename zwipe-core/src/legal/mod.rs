//! Shared legal copy.
//!
//! The privacy policy body lives here as a single HTML `const` so the mobile app
//! (`zwiper`) and the website (`zite`) render identical text from one source and
//! can never drift apart. It is pure data — a `&'static str` — so it adds no
//! dependencies and respects this crate's purity rules. Both apps render it with
//! `div { dangerous_inner_html: PRIVACY_POLICY_HTML }`; this is safe because the
//! copy is our own static text, never user input.
//!
//! The support `mailto:` contact line is intentionally NOT included here: a raw
//! `mailto:` anchor silently fails inside the mobile webview, so each app renders
//! its own contact line. Inline `https://` links work in both apps for free.

/// When the privacy policy was last revised, shown in each app's header.
pub const PRIVACY_LAST_UPDATED: &str = "June 2026";

/// The privacy policy body as an HTML fragment — headings, paragraphs, lists, and
/// inline `https://` links. Rendered via `dangerous_inner_html` in both apps. The
/// contact line (support `mailto:`) is rendered per-app and is not part of this.
pub const PRIVACY_POLICY_HTML: &str = r#"<h2>Overview</h2>
<p>Zwipe is a <a href="https://magic.wizards.com/en" target="_blank" rel="noopener noreferrer">Magic: The Gathering</a> deck builder for mobile. This policy describes what data we collect, how we use it, and your rights.</p>

<h2>Data We Collect</h2>
<ul>
<li><strong>Account data</strong>: email address, username, and a hashed password (never stored in plaintext).</li>
<li><strong>Deck data</strong>: the decks and card selections you create within the app.</li>
<li><strong>Session data</strong>: authentication tokens stored securely on your device.</li>
<li><strong>Usage analytics</strong>: how you interact with the app, including swiping activity, used in aggregate to improve the experience.</li>
</ul>
<p>We do not collect location data or device identifiers, or any data beyond what is required to operate and improve the app.</p>

<h2>How We Use Your Data</h2>
<ul>
<li>To authenticate your account and maintain sessions.</li>
<li>To store and sync your decks across devices.</li>
<li>To send transactional emails (email verification, password reset).</li>
<li>To analyze swiping and usage patterns in aggregate so we can improve the app.</li>
</ul>
<p>We do not sell, share, or use your data for advertising.</p>

<h2>Third-Party Services</h2>
<ul>
<li><strong><a href="https://scryfall.com" target="_blank" rel="noopener noreferrer">Scryfall</a></strong>: card data (names, images, oracle text) is sourced from the Scryfall API and stored on our servers. Your account data is never shared with Scryfall.</li>
<li><strong><a href="https://resend.com" target="_blank" rel="noopener noreferrer">Resend</a></strong>: transactional email delivery (verification and password reset emails). Your email address is passed to Resend solely to deliver these messages.</li>
<li><strong><a href="https://archidekt.com" target="_blank" rel="noopener noreferrer">Archidekt</a></strong>: when you import a deck from Archidekt, we request that deck's public card data from the Archidekt API. This only happens when you use the import feature, and no account data is shared with Archidekt.</li>
</ul>

<h2>Data Retention</h2>
<p>Your data is retained as long as your account exists. You can delete your account at any time from within the app, which permanently removes all associated data.</p>

<h2>Security</h2>
<p>Passwords are hashed with argon2. Refresh tokens are SHA-256 hashed before storage. All traffic is encrypted in transit via HTTPS. We do not have access to your plaintext password.</p>

<h2>Children</h2>
<p>Zwipe is not directed at children under 13. We do not knowingly collect data from children under 13.</p>

<h2>Fan Content</h2>
<p>Zwipe is unofficial Fan Content permitted under the <a href="https://company.wizards.com/en/legal/fancontentpolicy" target="_blank" rel="noopener noreferrer">Fan Content Policy</a>. Not approved/endorsed by Wizards. Portions of the materials used are property of Wizards of the Coast. ©Wizards of the Coast LLC.</p>"#;
