// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Mark, SEEDED } from "../../ideas/look-pens-model";
import { type Recording } from "../recording";

import {
  type AnnotationStyle,
  DEFAULT_STYLE,
  penStyle,
} from "./annotation-style";

// The annotations on the recording: where each points, when it shows and how
// it is drawn. The shared picture model's own look and swatch are left
// behind: here every annotation carries its own style.

export type TimedMark = Pick<Mark, "element" | "from" | "id" | "pen" | "to"> & {
  end: number;
  start: number;
  style: AnnotationStyle;
};

/** A mark placed on the picture, timed and drawn in its pen's style. */
export const timedMark = (
  { element, from, id, pen, to }: Mark,
  time: { end: number; start: number },
  style: AnnotationStyle,
): TimedMark => ({ element, from, id, pen, to, ...time, style });

/** The seeded annotations a recording places, timed as it places them: a
 * game or a camera has none. */
export const seededMarks = (recording: Recording): TimedMark[] =>
  SEEDED.flatMap((mark) => {
    const time = recording.markTimes[mark.id] as
      Recording["markTimes"][number] | undefined;
    return time ? [timedMark(mark, time, DEFAULT_STYLE)] : [];
  });

/** Moves when a mark shows, within the recording's `duration`; it keeps a
 * sliver at least, so one end cannot be dragged past the other. */
export const retimed = (
  marks: TimedMark[],
  {
    duration,
    end,
    id,
    start,
  }: Pick<TimedMark, "end" | "id" | "start"> & {
    duration: number;
  },
) => {
  const from = Math.max(0, Math.min(start, end - 0.3));
  const to = Math.min(duration, Math.max(end, from + 0.3));
  return marks.map((mark) =>
    mark.id === id ? { ...mark, end: to, start: from } : mark,
  );
};

/** Every annotation of the chosen one's kind, drawn as the chosen one is. */
export const matchedTo = (marks: TimedMark[], chosen: TimedMark) =>
  marks.map((mark) =>
    mark.pen === chosen.pen
      ? {
          ...mark,
          style: { ...mark.style, ...penStyle(mark.pen, chosen.style) },
        }
      : mark,
  );
