# Clips runbook

How to record the short phone clips for zwipe.net and the portfolio, and the longer TikTok cuts. Everything runs on the Android emulator against prod, signed in as the `test` account.

## 1. Emulator and build

The emulator is a Pixel 9a AVD (1080x2424, dpr 2.625). Setup is in `context/operations/android/emulator.md`.

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export PATH="$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export BACKEND_URL="https://api.zwipe.net"     # debug builds hit localhost without this

emulator -avd Pixel_9a -netdelay none -netspeed full &
git checkout --detach origin/release
zcripts/android/debug_apk.sh
adb install -r target/dx/zwiper/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
```

Debug builds only go on the emulator, never on a phone. Sign in as `test`.

To tap things, CSS px map to device px as `(x * 2.625, y * 2.625 + 142)`. The tap coordinates below are device px on this AVD.

## 2. Demo deck

Clips need a real-looking commander deck, not "test" names.

- For stats, playstyle and activity, use `isshin` (101 cards, Isshin, Two Heavens as One). These clips only read the deck.
- Swiping on Add changes a deck, so use a temporary one. Create deck, name it something real ("Atraxa Counters"), set Format to Commander and pick the commander (Atraxa, Praetors' Voice). Delete it afterwards: deck page, Danger zone, Delete.
- Put back anything else you touch. Deck page sections remember whether they are open (isshin's Budget is open), and Activity's Card roles chip goes back to Added.

## 3. Theme and demo mode

The theme sheet is under ☰ then the palette icon. Pick **Gruvbox** (dark). The account's normal theme is Kanagawa Dragon, so switch back when you are done.

```bash
adb shell settings put global sysui_demo_allowed 1
d() { adb shell am broadcast -a com.android.systemui.demo -e command "$@" >/dev/null; }
d enter; d clock -e hhmm 0941; d battery -e level 100 -e plugged false
d notifications -e visible false; d network -e wifi show -e level 4

# afterwards
d exit; adb shell settings put global sysui_demo_allowed 0
```

The app draws its own grey band over the status bar, so demo mode makes no visible difference: the top 142px record as flat grey with no clock or icons. The fix is to crop it off, along with the gesture bar at the bottom (y 2360 to 2400), giving 1080x2218. Keep both versions: full frame in the dated folder and cropped in `cropped/`.

## 4. Record

```bash
adb shell screenrecord --size 1080x2400 --bit-rate 12000000 /sdcard/c1.mp4 &
sleep 1          # a beat on the first frame, then start tapping
# ... taps and swipes (section 6) ...
adb shell pkill -INT screenrecord
sleep 2 && adb pull /sdcard/c1.mp4 raw/ && adb shell rm /sdcard/c1.mp4
```

Timing:

- Each clip is 5 to 8s. Start the recorder right before the first tap, and leave about 1s of hold at the end.
- Drive taps with a script (`adb shell input tap/swipe`). Hand-driven timing drifts.
- Swipe cards with `input swipe 400 1250 1050 1150 220` (right, add) and `input swipe 680 1250 30 1150 220` (left, skip), about 1.3s apart.
- Scrolls fling. `input swipe 540 2000 540 900 250` moves roughly a screen and a half. Dry-run each clip and screenshot it before recording.
- Check a take with a contact sheet: `ffmpeg -i raw/c1.mp4 -vf "fps=2,scale=216:-1,tile=8x2" -frames:v 1 sheet.png`

## 5. Trim and crop

screenrecord writes variable frame rate: there are no frames while the screen is still, and none after the last change. So convert to 30fps **before** trimming, then hold the last frame. `-ss` before `-i` drops the opening frame and shortens the clip.

```bash
# START = seconds to cut from the front, LEN = final length
ffmpeg -i raw/c1.mp4 -vf "fps=30,trim=start=START,setpts=PTS-STARTPTS,tpad=stop_mode=clone:stop_duration=3" \
  -t LEN -c:v libx264 -crf 16 -preset slow -pix_fmt yuv420p -movflags +faststart -an 1_swipe.mp4

# cropped copy: same command plus the crop
ffmpeg -i raw/c1.mp4 -vf "fps=30,trim=start=START,setpts=PTS-STARTPTS,tpad=stop_mode=clone:stop_duration=3,crop=1080:2218:0:142" \
  -t LEN -c:v libx264 -crf 16 -preset slow -pix_fmt yuv420p -movflags +faststart -an cropped/1_swipe.mp4
```

## 6. The four shots

Folder layout:

```
~/Developer/droid-clips/YYYY-MM-DD/
  1_swipe.mp4  2_stats.mp4  3_playstyle.mp4  4_activity.mp4   # 1080x2400
  cropped/     # same names, 1080x2218
  raw/         # untrimmed takes c1..c4.mp4
```

Starting state, recorded actions and trim for each shot (2026-11-08 takes):

| Shot | Start on | Recorded taps | Trim |
|---|---|---|---|
| 1_swipe | Temp deck, Cards, + (Add). Wait for the first card. | Swipe right, right, left, right, 1.3s apart | start 0.9, len 5.6 |
| 2_stats | isshin deck page, scrolled to top, sections folded | Tap Budget (540,1229), wait 1.3s, scroll `540 2000 540 1200 300`, tap Mana, wait 1.8s, tap Draw odds, then tap turn + (826,780) three times | start 0.7, len 6.6 |
| 3_playstyle | ☰, Playstyle (Colors open) | Tap Card roles (540,1218), wait 1.4s, tap Oracle tags (540,1839), wait 1.4s, scroll `540 2000 540 900 250` to Suggested commanders | start 1.0, len 5.4 |
| 4_activity | ☰, Activity (Activity panel open) | Tap Card roles (540,1970), wait 1.4s, tap Skipped chip (486,1104), wait 1.2s, scroll `540 2000 540 900 300` to Most skipped | start 0.9, len 5.2 |

These sections work as an accordion: opening one closes the others, so the page jumps. On the deck page, Mana and Draw odds move after the scroll, so read their position over CDP just before tapping. Leave Activity's Oracle tags out, because for `test` it only says "No tags stand out from other players yet".

## 7. TikTok cuts

These are vertical posts for TikTok, Reels and Shorts. Each takes 15 to 25s: about a 2s hook, then feature beats of 2 to 3s each, then the end card. The beat sheets are in `context/marketing/plans/`: start from `README.md`, then the `video_NN_*.md` files. Captions follow the plans' rules: sentence case, no em dashes, "Zwipe" capitalized.

1. Record each beat as its own short take (sections 4 and 5). One take per beat makes the edit a plain concat.
2. Turn each beat into 9:16 (1080x1920). There are two ways:
   - **Fit (default).** Scale the cropped take to 1920 tall and pad the sides with the theme background. This keeps the bottom toolbar (Add's swipe controls). ```bash ffmpeg -i cropped/beat1.mp4 -vf "scale=-2:1920,pad=1080:1920:(ow-iw)/2:0:color=0x282828,fps=30,format=yuv420p" -c:v libx264 -crf 16 -an tt/beat1.mp4 ```
   - **Crop.** Fills the frame but cuts the bottom 298px, so use it only when nothing important is down there. ```bash ffmpeg -i cropped/beat1.mp4 -vf "crop=1080:1920:0:0,fps=30,format=yuv420p" -c:v libx264 -crf 16 -an tt/beat1.mp4 ```
3. Make the end card a 3 to 4s clip from `context/marketing/video_end_card.png`, which is already 1080x1920:
   ```bash
   ffmpeg -loop 1 -t 3 -i context/marketing/video_end_card.png -vf "fps=30,format=yuv420p" -c:v libx264 -crf 16 tt/end.mp4
   ```
4. Concat. All parts share size, fps and codec, so `-c copy` works:
   ```bash
   printf "file '%s'\n" tt/beat1.mp4 tt/beat2.mp4 tt/beat3.mp4 tt/end.mp4 > tt/list.txt
   ffmpeg -f concat -safe 0 -i tt/list.txt -c copy tiktok_video04.mp4
   ```
5. Captions and VO go on in the editor or app you post from. Keep the source cut without text so it can be reused.

## 8. Teardown

- Delete the temp deck and check the deck count is back where it started.
- Switch the theme back to Kanagawa Dragon, exit demo mode, and remove the `/sdcard/*.mp4` files.
- Put back any deck sections or chips you changed (section 2).

## 9. Publishing to zwipe.net and the portfolio

(To fill in: thinker is setting this up. Cover where the files go, which version to use (full, cropped or 9:16), the encode or size limits, and how to deploy.)
