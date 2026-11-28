// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import { decibelLevel } from "../../../primitives/audio-meter/decibel-level";
import { useMicrophoneMute } from "../../microphone/use-microphone";
import { useRecordingMonitor } from "../../recording-controls/use-recording-monitor";
import { getNotesIslandState, runNotesIslandAction } from "../api";

import type { NotesMicrophone } from "./notes-island";
import type { NotesIslandAction } from "../../../bindings/NotesIslandAction";
import type { NotesIslandState } from "../../../bindings/NotesIslandState";

/**
 * Notes as the island shows them: `state` as last sent, kept after they go
 * so the island tucks away as it was; `null` until they have first been
 * up. `microphone` is the microphone
 * the notes hear, its level read while they are taken.
 */
export function useNotesIsland(): {
  microphone: NotesMicrophone;
  onAction: (action: NotesIslandAction) => void;
  state: NotesIslandState | null;
} {
  const [state, setState] = useState<NotesIslandState | null>(null);
  const [last, setLast] = useState<NotesIslandState | null>(null);
  if (state && state !== last) setLast(state);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    void getNotesIslandState().then((current) => {
      if (!disposed && !isToldRef.current) setState(current);
    });
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("notes://island", (payload) => {
    isToldRef.current = true;
    setState(payload as NotesIslandState | null);
  });

  // The notes hear on the recording's monitor while no recording runs,
  // when the recording island is not reading it (`start` in notes.rs).
  const monitor = useRecordingMonitor(state?.phase === "taking");
  const mute = useMicrophoneMute();

  return {
    microphone: {
      ...mute,
      level: mute.isMuted ? 0 : decibelLevel(monitor.microphoneDecibels),
    },
    onAction: (action) => {
      runNotesIslandAction(action).catch((cause: unknown) => {
        console.error(`Could not run the notes' ${action}`, cause);
      });
    },
    state: state ?? last,
  };
}
