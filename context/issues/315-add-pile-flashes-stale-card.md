# 315. Add's pile flashes a stale card after applying a filter

State: reported | Owner: - | PR: - | Build seen: 1.18.14

## Report
Droid, 2026-11-08, Android app (prod): on Add, applying a filter shows one card for about 0.3s (Deadly Rollick), then a different top card (Drakuseth) replaces it. Same in an earlier take (Abrade, then Rakdos Charm), so it isn't new. Likely the old pile's next card renders before the filtered pile arrives. Footage: ~/Developer/video-factory/videos/oracle-tags/footage/raw/o4.mp4 at ~9.6s.
