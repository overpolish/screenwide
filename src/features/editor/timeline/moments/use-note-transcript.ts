// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import {
  getRecordingMoments,
  listenToNoteTranscripts,
  NoteTranscript,
} from "./recording-moments";

/**
 * The `moment`th moment's transcript as it moves on. The panel window is
 * handed what the timeline knew when the note was opened, so it follows the
 * transcript itself rather than waiting to be told. What it has read is kept
 * against the note it read it for, so a panel moved on to another note
 * starts from that note's own.
 */
export function useNoteTranscript(
  artifactId: number,
  moment: number,
  opened: NoteTranscript | null,
) {
  const key = `${artifactId.toString()}:${moment.toString()}`;
  const [read, setRead] = useState<{
    key: string;
    transcript: NoteTranscript | null;
  } | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    let current = true;
    let stop: (() => void) | undefined;
    void listenToNoteTranscripts(() => {
      getRecordingMoments(artifactId)
        .then((moments) => {
          const note = moments.find((each) => each.index === moment)?.note;
          if (current) setRead({ key, transcript: note?.transcript ?? null });
        })
        .catch(() => undefined);
    })
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch(() => undefined);
    return () => {
      current = false;
      stop?.();
    };
  }, [artifactId, key, moment]);

  return read?.key === key ? read.transcript : opened;
}
