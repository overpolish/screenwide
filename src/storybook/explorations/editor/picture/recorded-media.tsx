// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { type RecordedMedia } from "../recording";

// A real recording's own picture: its movie, held at the playhead, and a
// frame from its sheet for a thumbnail. The movie is only ever sought, never
// played, so it shows whatever moment the editor is at, squeezed or not.

export function RecordedVideo({
  media,
  seconds,
}: {
  media: RecordedMedia;
  seconds: number;
}) {
  const ref = useRef<HTMLVideoElement>(null);
  useEffect(() => {
    const video = ref.current;
    if (video && Math.abs(video.currentTime - seconds) > 0.05)
      video.currentTime = seconds;
  }, [seconds]);
  return (
    <video
      className="size-full bg-black object-cover"
      muted
      playsInline
      preload="auto"
      ref={ref}
      src={media.video}
    />
  );
}

export function RecordedThumbnail({
  duration,
  media,
  seconds,
}: {
  duration: number;
  media: RecordedMedia;
  seconds: number;
}) {
  const sheet = media.thumbnails;
  if (!sheet) return <span className="block size-full bg-black" />;
  const frames = Math.max(1, Math.ceil(duration / sheet.every));
  const rows = Math.ceil(frames / sheet.columns);
  const frame = Math.min(frames - 1, Math.floor(seconds / sheet.every));
  const column = frame % sheet.columns;
  const row = Math.floor(frame / sheet.columns);
  return (
    <span
      className="block size-full bg-black bg-no-repeat"
      style={{
        backgroundImage: `url(${sheet.url})`,
        backgroundPosition: `${String((column / Math.max(sheet.columns - 1, 1)) * 100)}% ${String((row / Math.max(rows - 1, 1)) * 100)}%`,
        backgroundSize: `${String(sheet.columns * 100)}% ${String(rows * 100)}%`,
      }}
    />
  );
}
