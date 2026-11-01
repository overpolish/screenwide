// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";

import { getRecordingTranscript } from "../../recording/transcript/transcript-api";
import { useTranscriptRevision } from "../../recording/transcript/transcript-revision";
import { createRecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { useRecordingKeyboardTimeline } from "../keyboard/use-recording-keyboard-timeline";
import { type RecordingMoment } from "../moments/recording-moments";

import {
  getRecordingActivity,
  type RecordingActivity,
} from "./recording-activity-api";
import { type RecordingReadings } from "./recording-beats";

import type { RecordingTranscriptWord } from "../../../../bindings/RecordingTranscriptWord";

// What the recording open in the editor holds to find its beats in, read
// from the app: its activity, read once; its words, read again whenever
// the transcript is made again; its shortcuts, read as recorded, before any
// edit, so they stay where they were pressed; and its moments, which the
// band reads already.

const NO_ACTIVITY: RecordingActivity = { clicks: [], picture: [], typing: [] };
const NO_WORDS: RecordingTranscriptWord[] = [];

/** One answer per recording, cleared while another is read. */
function useLoaded<T>(
  artifactId: number,
  load: (artifactId: number) => Promise<T>,
  { none, revision = 0 }: { none: T; revision?: number },
) {
  const [loaded, setLoaded] = useState<{ artifactId: number; value: T } | null>(
    null,
  );
  useEffect(() => {
    let current = true;
    load(artifactId)
      .then((value) => {
        if (current) setLoaded({ artifactId, value });
      })
      .catch((cause: unknown) => {
        console.error("Could not read what the recording holds", cause);
      });
    return () => {
      current = false;
    };
  }, [artifactId, load, revision]);
  return loaded?.artifactId === artifactId ? loaded.value : none;
}

const loadWords = (artifactId: number) =>
  getRecordingTranscript(artifactId).then(
    (transcript) => transcript?.words ?? NO_WORDS,
  );

export function useRecordingStripReadings({
  artifactId,
  durationMs,
  hasKeyboardData,
  moments,
}: {
  artifactId: number;
  durationMs: number;
  hasKeyboardData: boolean;
  moments: RecordingMoment[];
}): RecordingReadings {
  const activity = useLoaded(artifactId, getRecordingActivity, {
    none: NO_ACTIVITY,
  });
  const transcript = useTranscriptRevision();
  const words = useLoaded(artifactId, loadWords, {
    none: NO_WORDS,
    revision: transcript.artifactId === artifactId ? transcript.revision : 0,
  });
  // As recorded: shortcuts deleted since are left out by the strip itself.
  const asRecorded = useMemo(
    () => createRecordingTimelineEdit(artifactId),
    [artifactId],
  );
  const keyboard = useRecordingKeyboardTimeline({
    artifactId,
    edit: asRecorded,
    enabled: hasKeyboardData,
    sourceDurationMs: durationMs,
  });
  return useMemo(() => {
    const seconds = (ms: number) => ms / 1000;
    return {
      clicks: activity.clicks.map(({ count, ms }) => ({
        at: seconds(ms),
        count,
      })),
      duration: seconds(durationMs),
      moments: moments.map(({ color, index, name, sourceMs }) => ({
        at: seconds(sourceMs),
        color,
        index,
        name,
      })),
      picture: activity.picture.map(({ endMs, startMs, state }) => ({
        end: seconds(endMs),
        start: seconds(startMs),
        state,
      })),
      shortcuts: keyboard.items.map(({ endMs, id, label, startMs }) => ({
        at: seconds(startMs),
        end: seconds(endMs),
        id: String(id),
        keys: label.split(" "),
      })),
      typing: activity.typing.map(seconds),
      words: words.map(({ endMs, startMs, text }) => ({
        end: seconds(endMs),
        start: seconds(startMs),
        text,
      })),
    };
  }, [activity, durationMs, keyboard.items, moments, words]);
}
