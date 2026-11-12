// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import type { RecordingTimelineEdit } from "../../../../bindings/RecordingTimelineEdit";

// The songs and sound effects are laid out for the preview as it starts
// playing, with whether the songs make way for the voice: Studio ducking on,
// and the microphone among the tracks heard. A change to any of that while
// it plays is heard at once: playback starts again where it is, with it.

type Player = { isPlaying: boolean; play: () => void };

export function useSoundTrackReplay(
  edit: RecordingTimelineEdit,
  /** The audio tracks heard, by stream. */
  heardStreams: readonly number[] | undefined,
  player: Player,
) {
  const playerRef = useRef(player);
  playerRef.current = player;
  const hasClips = (edit.audioClips?.length ?? 0) > 0;
  const replay = () => {
    if (playerRef.current.isPlaying) playerRef.current.play();
  };
  const heard = [
    edit.audioClips,
    edit.mutedAudio,
    hasClips ? (heardStreams ?? []).join(",") : "",
  ] as const;
  const heardRef = useRef(heard);
  useEffect(() => {
    const was = heardRef.current;
    heardRef.current = heard;
    if (was.some((each, index) => each !== heard[index])) replay();
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- only a change to what is heard plays again.
  }, heard);
  useEffect(() => {
    if (!hasClips) return;
    let current = true;
    let stop: (() => void) | undefined;
    listen("editor://studio-ducking", replay)
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow Studio ducking", cause);
      });
    return () => {
      current = false;
      stop?.();
    };
  }, [hasClips]);
}
