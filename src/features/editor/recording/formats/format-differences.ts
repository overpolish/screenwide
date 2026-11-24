// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  CAPTIONS_ID,
  type FormatContext,
  isShownIn,
  layoutKeys,
  shownFormats,
} from "./format-scope";

import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { FormatScope } from "../../../../bindings/FormatScope";
import type { RecordingAnnotationClip } from "../../../../bindings/RecordingAnnotationClip";
import type { RecordingAudioClip } from "../../../../bindings/RecordingAudioClip";
import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

/** What a format does differently from the shared edit. */
export type FormatDifferences = {
  /** What this format leaves out, as it plays in the others: the
   * annotations, songs and sound effects the strip draws as ghosts, the
   * scenes standing plain here, and by id those and the caption track
   * (`CAPTIONS_ID`), each with the formats it shows in. */
  hidden: {
    annotations: readonly RecordingAnnotationClip[];
    audio: readonly RecordingAudioClip[];
    scenes: readonly RecordingSceneClip[];
    shownIn: Readonly<Record<string, readonly string[]>>;
  };
  /** Each scene, annotation, song or sound effect shown here and added
   * alone or laid out its own way, by id, for their badges. */
  ids: readonly string[];
  /** Of them, those laid out their own way, which can be given the shared
   * layout back. */
  laidOut: readonly string[];
};

export const NO_DIFFERENCES: FormatDifferences = {
  hidden: { annotations: [], audio: [], scenes: [], shownIn: {} },
  ids: [],
  laidOut: [],
};

type Formats = { layouts?: Record<string, unknown>; showIn?: FormatScope };

/** Every scene, annotation, song and sound effect of the shared edit,
 * with where it plays and what the formats do differently with it. */
export const formatItems = (
  edit: RecordingTimelineEdit,
  edits: FormatEdits,
): { endMs: number; id: string; startMs: number; formats?: Formats }[] => [
  ...(edit.sceneClips ?? []).map(({ endMs, id, startMs }) => ({
    endMs,
    formats: edits.scenes?.[id],
    id,
    startMs,
  })),
  ...(edit.annotationClips ?? []).map(({ annotation, endMs, startMs }) => ({
    endMs,
    formats: edits.annotations?.[annotation.id],
    id: annotation.id,
    startMs,
  })),
  ...(edit.audioClips ?? []).map(({ endMs, id, startMs }) => ({
    endMs,
    formats: edits.audio?.[id],
    id,
    startMs,
  })),
];

export function formatDifferences(
  edit: RecordingTimelineEdit,
  edits: FormatEdits | undefined,
  context: FormatContext,
): FormatDifferences {
  if (!edits) return NO_DIFFERENCES;
  const keys = layoutKeys(context);
  const isLaidOut = (formats: Formats | undefined) =>
    keys.some((key) => formats?.layouts?.[key] !== undefined);
  const items = formatItems(edit, edits);
  // What this format leaves out is drawn as a ghost rather than marked: a
  // scene left out stands as a plain one, the rest are not here at all.
  const shown = items.filter((item) =>
    isShownIn(item.formats?.showIn, context),
  );
  const found = shown.filter(
    (item) => item.formats?.showIn !== undefined || isLaidOut(item.formats),
  );
  const isHere = (showIn: FormatScope | undefined) =>
    isShownIn(showIn, context);
  const shownIn: Record<string, readonly string[]> = {};
  for (const item of items)
    if (!isHere(item.formats?.showIn))
      shownIn[item.id] = shownFormats(item.formats?.showIn, context);
  const captions = edits.captions?.showIn;
  if (edit.captions?.enabled && !isHere(captions))
    shownIn[CAPTIONS_ID] = shownFormats(captions, context);
  return {
    hidden: {
      annotations: (edit.annotationClips ?? []).filter(
        (clip) => clip.annotation.id in shownIn,
      ),
      audio: (edit.audioClips ?? []).filter((clip) => clip.id in shownIn),
      scenes: (edit.sceneClips ?? []).filter((clip) => clip.id in shownIn),
      shownIn,
    },
    ids: found.map(({ id }) => id),
    laidOut: found
      .filter((item) => isLaidOut(item.formats))
      .map(({ id }) => id),
  };
}
