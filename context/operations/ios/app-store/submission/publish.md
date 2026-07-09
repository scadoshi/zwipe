# Publish to the App Store (upload + submit)

Takes the signed `Zwipe.ipa` from [build.md](build.md) and gets it into review.

---

## 1. Upload via Transporter

**Do NOT use `xcrun altool`** — it is deprecated and causes metadata parsing errors
that can trigger false "beta Xcode" rejections. See [debugging.md](debugging.md)
for details.

**Do NOT use `xcrun iTMSTransporter`** — it expects `.itmsp` directories, not `.ipa` files.

1. Open **Transporter** (Mac App Store, free, by Apple)
2. Sign in with your Apple ID if prompted
3. Drag `~/Developer/zwipe/Zwipe.ipa` into the window
4. Click **Deliver** — validates and uploads in one step
5. Wait for "Upload Successful" confirmation

The build will appear in App Store Connect after 5–10 minutes.

### Fallback: altool (deprecated — use only if Transporter is unavailable)

```bash
xcrun altool --validate-app -f ~/Developer/zwipe/Zwipe.ipa -t ios \
  --apiKey C2L47TDDPV --apiIssuer 644db668-17b6-4d50-ac1a-70f8ea838d0d

xcrun altool --upload-app -f ~/Developer/zwipe/Zwipe.ipa -t ios \
  --apiKey C2L47TDDPV --apiIssuer 644db668-17b6-4d50-ac1a-70f8ea838d0d
```

API key file: `~/.private_keys/AuthKey_C2L47TDDPV.p8`

## 2. Submit

1. App Store Connect → your app → build appears after 5–10 min
2. Create new version if needed (click **+** next to iOS App)
3. Select the build
4. Export Compliance → **No**
5. **Submit for Review**

---

Listing copy (What's New, description, keywords) lives in
[form_fields.md](form_fields.md). Beta distribution is in
[testflight.md](testflight.md).
