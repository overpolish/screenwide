// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { streamRecordingTimelineThumbnails } from "../../api";
import { useCameraEffectsRevision } from "../../recording/camera/use-camera-effects-revision";
import {
  RecordingTimelineThumbnails,
  RecordingVideoTrackId,
} from "../../types";

const HEADER_LENGTH = 20;
const HEADER_MARKER = 0x4854434f;
const HEADER_VERSION = 1;
const THUMBNAIL_COUNT = 24;
const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47];

const emptyThumbnails = (): RecordingTimelineThumbnails => ({
  camera: [],
  primary: [],
});

type DecodedThumbnail = {
  bytes: ArrayBuffer;
  count: number;
  index: number;
  track: RecordingVideoTrackId;
};

const decodeThumbnail = (payload: ArrayBuffer): DecodedThumbnail | null => {
  if (payload.byteLength <= HEADER_LENGTH) return null;
  const header = new DataView(payload, 0, HEADER_LENGTH);
  if (
    header.getUint32(0, true) !== HEADER_MARKER ||
    header.getUint32(4, true) !== HEADER_VERSION
  )
    return null;
  const rawTrack = header.getUint32(8, true);
  const index = header.getUint32(12, true);
  const count = header.getUint32(16, true);
  if (rawTrack > 1 || count === 0 || index >= count) return null;
  return {
    bytes: payload.slice(HEADER_LENGTH),
    count,
    index,
    track: rawTrack === 0 ? "primary" : "camera",
  };
};

export function useRecordingTimelineThumbnails({
  artifactId,
  isEnabled,
}: {
  artifactId: number;
  isEnabled: boolean;
}) {
  const [thumbnails, setThumbnails] =
    useState<RecordingTimelineThumbnails>(emptyThumbnails);
  // The camera's thumbnails show its effects, so they are made again when
  // those change.
  const effectsRevision = useCameraEffectsRevision(artifactId);
  // Every URL made and not yet let go. A thumbnail made again replaces the
  // one shown only once it arrives, so the strip never blanks between.
  const urlsRef = useRef(new Set<string>());

  useEffect(() => {
    const made = urlsRef.current;
    return () => {
      for (const url of made) URL.revokeObjectURL(url);
      made.clear();
    };
  }, []);

  useEffect(() => {
    if (!isEnabled) return;
    let disposed = false;
    const made = urlsRef.current;
    const channel = new Channel<ArrayBuffer>();
    channel.onmessage = (payload) => {
      if (disposed) return;
      const thumbnail = decodeThumbnail(payload);
      if (!thumbnail) return;
      // A see-through camera arrives as a PNG, every other thumbnail as a
      // JPEG.
      const isPng = new Uint8Array(thumbnail.bytes, 0, 4).every(
        (byte, index) => byte === PNG_SIGNATURE[index],
      );
      const url = URL.createObjectURL(
        new Blob([thumbnail.bytes], {
          type: isPng ? "image/png" : "image/jpeg",
        }),
      );
      made.add(url);
      setThumbnails((current) => {
        const track = Array.from(
          { length: thumbnail.count },
          (_, index) =>
            current[thumbnail.track][index] ?? {
              id: `${thumbnail.track}-${index.toString()}`,
              url: null,
            },
        );
        const previous = track[thumbnail.index].url;
        if (previous) {
          URL.revokeObjectURL(previous);
          made.delete(previous);
        }
        track[thumbnail.index] = { ...track[thumbnail.index], url };
        return { ...current, [thumbnail.track]: track };
      });
    };
    void streamRecordingTimelineThumbnails(
      artifactId,
      THUMBNAIL_COUNT,
      channel,
    ).catch((cause: unknown) => {
      if (!disposed)
        console.error("Could not prepare timeline thumbnails", cause);
    });
    return () => {
      disposed = true;
    };
  }, [artifactId, effectsRevision, isEnabled]);

  return thumbnails;
}
