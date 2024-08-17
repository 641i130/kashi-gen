#!/bin/sh
rm -rf output/* *.mp4
cargo run --release && \
ffmpeg -r 30 -i output/frame_%06d.png -vf "fps=30,format=yuv420p" output.mp4 && \
ffmpeg -i output.mp4 -i song.mp3 -i lyrics.srt -c:s mov_text c:v copy -c:a aac -strict experimental -map 0:v:0 -map 1:a:0 output_combined.mp4 && \
mpv output_combined.mp4
