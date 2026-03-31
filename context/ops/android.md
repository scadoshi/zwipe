# Android Development Setup

## Prerequisites

Install Android Studio (includes SDK, NDK, emulator):

```bash
brew install --cask android-studio
```

Open Android Studio, complete the setup wizard. Then install the NDK:
- Settings > Languages & Frameworks > Android SDK > SDK Tools tab
- Check "NDK (Side by side)" > Apply

## Environment

Add to `~/.zshrc`:

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/<version>"
```

Find the version with `ls ~/Library/Android/sdk/ndk/`.

## Build and serve

```bash
cd ~/Developer/zwipe/zwiper

# Serve with hot reload (requires running emulator)
BACKEND_URL=https://api.zwipe.net dx serve --platform android

# Build only
BACKEND_URL=https://api.zwipe.net dx build --platform android
```

## Emulator

Create a virtual device in Android Studio:
- Tools > Device Manager (or "More Actions" > "Virtual Device Manager" from welcome screen)
- Create Virtual Device > pick a phone (e.g. Pixel 9) > download a system image > Finish
- Hit play to launch

Or via CLI:

```bash
# List available emulators
$ANDROID_HOME/emulator/emulator -list-avds

# Launch one
$ANDROID_HOME/emulator/emulator -avd <name>
```

`dx serve` will detect the running emulator and deploy automatically.

## Notes

- Same Rust codebase as iOS — one `zwiper/` directory, different build targets
- `BACKEND_URL` is baked in at compile time via `env!()`
- Android uses WebView for rendering (vs iOS WKWebView) — CSS rendering may differ slightly
- Known issues tracked in `context/status/todo.md` under Android section
