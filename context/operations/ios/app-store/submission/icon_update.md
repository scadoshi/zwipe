# Update the App Icon

Short guide for refreshing the iOS app icon without a full Rust rebuild. Use this when only the icon changed — same Rust code, same Info.plist, just new artwork.

For a fresh release build (icon + Rust changes), follow [build.md](build.md) and let step 3 swap the icons as part of the normal flow.

---

## Pre-flight

You need a single **1024×1024 PNG master** with no rounded corners (iOS rounds at render time) and a solid background. Transparency must be stripped before submission — Apple rejects alpha on the marketing icon.

### Generating the master (the ASCII Z logo)

The logo is the project's ASCII "Z" colorized and exported via the **asciier**
tool — https://github.com/scadoshi/asciier.git (open `main.html` in a browser):

- Pick the matching theme preset (**Gruvbox Dark** → cream `#ebdba2` on `#282828`)
  so the icon matches the app's default theme.
- Use the **JetBrains Mono** font at **line height 0.9×** — matches the in-app
  `.logo` rendering (`zwiper/assets/main.css`), so the block glyphs line up the
  same way they do on the home screen.
- Set the **size scale to 1.6×** — that's the sweet spot for filling the icon
  tile. 1.5× sits a touch small, 1.7× a touch large; **build 42 at 1.6× fit best**.
- Make sure the pasted ASCII has **no leading/trailing blank lines**, or the
  glyph renders off-center (vertically pushed) in the square.
- Export at 1024×1024, then run the steps below to flatten + resize.

Tools used: `magick` (ImageMagick, via Homebrew) and `sips` (built-in).

```bash
brew install imagemagick   # one-time
```

---

## 1. Prep the master + sizes

```bash
SRC=/path/to/new-master-1024.png
DEST=~/Developer/zwipe/zwiper/assets/favicon

# Flatten alpha against gruvbox bg (#282828).
# If your master uses a different background, set it here.
magick "$SRC" -background "#282828" -alpha remove -alpha off "$DEST/icon-1024.png"

# Resize down to every size iOS needs.
for SIZE in 40 60 80 87 120 180; do
  magick "$DEST/icon-1024.png" -resize ${SIZE}x${SIZE} "$DEST/icon-${SIZE}.png"
done

# Verify alpha is gone and dimensions are right.
for F in icon-40 icon-60 icon-80 icon-87 icon-120 icon-180 icon-1024; do
  sips -g pixelWidth -g pixelHeight -g hasAlpha "$DEST/$F.png" | grep -E "icon|pixel|Alpha"
done
# Every hasAlpha should read "no".
```

### Background-color drift

If the source PNG was rendered against a slightly off color (e.g. `#272727` instead of gruvbox `#282828`), recolor at the flatten step:

```bash
magick "$SRC" -background "#282828" -alpha remove -alpha off \
  -fuzz 1% -fill "#282828" -opaque "#272727" \
  "$DEST/icon-1024.png"
```

The small `-fuzz 1%` keeps anti-aliased edges intact while replacing solid background pixels.

### Sanity-check the colors

```bash
magick "$DEST/icon-1024.png" -format "%[pixel:p{4,4}]" info:        # corner = bg
magick "$DEST/icon-1024.png" -format "%[pixel:p{512,500}]" info:    # center = fg
```

---

## 2. Repack into the existing `.app` (no Rust rebuild)

This assumes you ran `dx build --release --platform ios` recently and `~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app` already exists. If not, do that first (step 1 of `build.md`).

```bash
APP=~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app

# Stage the icon catalog (same Contents.json as build.md step 3).
rm -rf /tmp/Assets.xcassets
mkdir -p /tmp/Assets.xcassets/AppIcon.appiconset
cat > /tmp/Assets.xcassets/AppIcon.appiconset/Contents.json << 'EOF'
{
  "images": [
    {"size":"20x20","idiom":"iphone","scale":"2x","filename":"icon-40.png"},
    {"size":"20x20","idiom":"iphone","scale":"3x","filename":"icon-60.png"},
    {"size":"29x29","idiom":"iphone","scale":"2x","filename":"icon-60.png"},
    {"size":"29x29","idiom":"iphone","scale":"3x","filename":"icon-87.png"},
    {"size":"40x40","idiom":"iphone","scale":"2x","filename":"icon-80.png"},
    {"size":"40x40","idiom":"iphone","scale":"3x","filename":"icon-120.png"},
    {"size":"60x60","idiom":"iphone","scale":"2x","filename":"icon-120.png"},
    {"size":"60x60","idiom":"iphone","scale":"3x","filename":"icon-180.png"},
    {"size":"1024x1024","idiom":"ios-marketing","scale":"1x","filename":"icon-1024.png"}
  ],
  "info":{"version":1,"author":"xcode"}
}
EOF
cat > /tmp/Assets.xcassets/Contents.json << 'EOF'
{"info":{"version":1,"author":"xcode"}}
EOF
cp ~/Developer/zwipe/zwiper/assets/favicon/icon-{40,60,80,87,120,180,1024}.png \
   /tmp/Assets.xcassets/AppIcon.appiconset/

# Recompile Assets.car (overwrites the existing one inside the .app).
actool --compile $APP --platform iphoneos --minimum-deployment-target 16.0 \
  --app-icon AppIcon --output-partial-info-plist /tmp/assetcatalog_generated_info.plist \
  /tmp/Assets.xcassets
```

---

## 3. Optional: bump build number

If the `.app` was already signed and uploaded to App Store Connect, bump the build number before re-uploading — Apple rejects duplicate build numbers.

```bash
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion <NEW_BUILD_NUMBER>" $APP/Info.plist
```

If you're iterating before the first Deliver click, you can stay on the same build number — Transporter happily replaces the staged item.

---

## 4. Re-sign + re-package

Changing `Assets.car` (or any file inside the `.app`) invalidates the existing signature. Re-sign and re-zip:

```bash
codesign --force --sign "<HASH-OR-NAME>" \
  --entitlements ~/Developer/zwipe/zwiper/Entitlements-Release.plist $APP

cd ~/Developer/zwipe
rm -f Zwipe.ipa
mkdir -p Payload
cp -r target/dx/zwipe/release/ios/Zwipe.app Payload/
zip -qr Zwipe.ipa Payload
rm -rf Payload

# Verify the new signature.
codesign -dvv $APP | grep -E "Authority|Signed Time"
```

---

## 5. Confirm the icon is actually in the bundle

Tools like Transporter render the icon thumbnail at ~70px, which makes chunky pixel-art icons look washed out at that size. Don't trust the preview for color verification — inspect the catalog directly:

```bash
xcrun assetutil --info $APP/Assets.car | grep -A 5 -i "AppIcon" | head -40
```

You should see a rendition for `icon-1024.png` with `Opaque: true` and `PixelWidth/Height: 1024`. The SHA1Digest changes whenever the source PNG changes.

To eyeball the actual rendered icon outside Transporter, just open the source PNG:

```bash
open ~/Developer/zwipe/zwiper/assets/favicon/icon-1024.png
```

---

## 6. Upload

Same as [publish.md](publish.md) step 1 — drag `~/Developer/zwipe/Zwipe.ipa` into Transporter and click Deliver.

---

## Cost comparison

Full flow (rebuild Rust + repack):  ~45–60s
Icon-only repack (this doc):         ~8s

Worth using the short flow whenever the only change is artwork.
