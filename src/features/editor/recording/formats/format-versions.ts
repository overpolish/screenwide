// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { formatItems } from "./format-differences";
import {
  CAPTIONS_ID,
  type FormatContext,
  isShownIn,
  layoutKeys,
} from "./format-scope";

import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

// The versions a recording comes out in: formats that include the same
// things and lay them out the same way are one version, as TikTok and
// Shorts are with nothing set apart. Each is drawn as a line over the
// strip's overview, marked only where versions differ: where something
// shows in some versions and not others, and where something is laid out
// its own way in a version's shape. What every version has alike is left
// unmarked, so a gap where another line has a mark is what this version
// lacks.

/** One mark on a version's line, in played seconds; one with no end runs
 * to the end of the recording, as the caption track does. */
export type VersionMark = { end: number | null; id: string; start: number };

export type FormatVersion = {
  /** Its formats, by id, in the order the bar shows them. */
  formats: readonly string[];
  /** Whether its screen is set its own way, for the whole recording: shown
   * by its name rather than as a mark, which would cover the others. */
  isPlaced: boolean;
  marks: readonly VersionMark[];
};

/** Stands for a version's own screen among what sets it apart. */
const PLACEMENT_ID = "placement";

/** The recording's versions among `formats`, each read in its own context
 * (`contextOf`); none while every format comes out the same. A version
 * whose shape `isPlaced` is one apart. */
export function formatVersions({
  contextOf,
  edit,
  edits,
  formats,
  isPlaced = () => false,
}: {
  contextOf: (id: string) => FormatContext;
  edit: RecordingTimelineEdit;
  edits: FormatEdits;
  formats: readonly string[];
  isPlaced?: (shape: string) => boolean;
}): FormatVersion[] {
  const items = formatItems(edit, edits);
  const captions = edit.captions?.enabled
    ? { formats: edits.captions, id: CAPTIONS_ID }
    : null;
  const read = formats.map((id) => {
    const context = contextOf(id);
    const shown = (showIn: Parameters<typeof isShownIn>[0]) =>
      isShownIn(showIn, context);
    const included = items
      .filter((item) => shown(item.formats?.showIn))
      .map((item) => item.id);
    if (captions && shown(captions.formats?.showIn)) included.push(CAPTIONS_ID);
    const keys = layoutKeys(context);
    const laidOut = items
      .filter((item) =>
        keys.some((key) => item.formats?.layouts?.[key] !== undefined),
      )
      .map((item) => item.id);
    if (isPlaced(context.shape)) laidOut.push(PLACEMENT_ID);
    return { id, included, laidOut };
  });
  // Something every format includes, laid out alike everywhere, is no
  // difference.
  const everywhere = (item: string) =>
    read.every((format) => format.included.includes(item));
  const groups = new Map<string, (typeof read)[number][]>();
  for (const format of read) {
    const key = JSON.stringify([format.included, format.laidOut]);
    groups.set(key, [...(groups.get(key) ?? []), format]);
  }
  if (groups.size < 2) return [];
  return [...groups.values()].map((group) => {
    const [{ included, laidOut }] = group;
    const marked = (item: string) =>
      (included.includes(item) && !everywhere(item)) || laidOut.includes(item);
    return {
      formats: group.map(({ id }) => id),
      isPlaced: laidOut.includes(PLACEMENT_ID),
      marks: [
        ...items
          .filter((item) => marked(item.id))
          .map((item) => ({
            end: item.endMs / 1000,
            id: item.id,
            start: item.startMs / 1000,
          })),
        ...(captions && marked(CAPTIONS_ID)
          ? [{ end: null, id: CAPTIONS_ID, start: 0 }]
          : []),
      ],
    };
  });
}
