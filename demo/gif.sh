#!/usr/bin/env bash
# Turn one recorded act into a GIF for the README.
#
#   ./demo/gif.sh demo/out/01-basics.mp4          -> demo/demo.gif
#   ./demo/gif.sh demo/out/03-pubsub.mp4 pubsub   -> demo/pubsub.gif
#
# GitHub strips <video> from READMEs, so an inline demo has to be a GIF. Two
# passes: generate a palette, then map to it.
#
# Card art is full-color illustration, which GIF handles far worse than terminal
# text. Trimming to the seconds that actually move is the biggest lever: a static
# card on screen costs about as much as a swipe.
#
#   SS=33 T=7 ./demo/gif.sh assets/3_add_cards.mp4

set -euo pipefail
cd "$(dirname "$0")/.."

SRC="${1:?usage: gif.sh <clip.mp4> [name]}"
OUT="demo/${2:-demo}.gif"
FPS="${FPS:-10}"
WIDTH="${WIDTH:-340}"   # portrait phone capture, so narrow rather than the 900 a terminal wants
SS="${SS:-0}"           # trim: card art is expensive, and a static dwell costs as much as motion
T="${T:-}"
PAL=$(mktemp -t gifpal).png

TRIM=(-ss "$SS"); [ -n "$T" ] && TRIM+=(-t "$T")

ffmpeg -v error -y "${TRIM[@]}" -i "$SRC" \
  -vf "fps=$FPS,scale=$WIDTH:-1:flags=lanczos,palettegen=stats_mode=diff" "$PAL"

# bayer dithering keeps flat terminal backgrounds from developing noise, which
# the default error-diffusion adds and which costs a lot of bytes in a GIF.
ffmpeg -v error -y "${TRIM[@]}" -i "$SRC" -i "$PAL" \
  -lavfi "fps=$FPS,scale=$WIDTH:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=3" \
  "$OUT"

rm -f "$PAL"
printf '%s  %s\n' "$OUT" "$(du -h "$OUT" | cut -f1)"
