// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ToolPanelStripItem } from "../../tools/tool-panel-strip";
import { type RecordingMoment } from "../moments/recording-moments";
import { baseOf } from "../strip/strip-part";
import { type StripSelection } from "../strip/strip-selection";

import { type RecordingBeat } from "./recording-beats";
import { type RecordingStripLulls } from "./recording-strip-lulls";

// What the island of what is chosen on the strip shows, read from the
// strip's choice: a beat as it was found, a moment with its voice note, a
// lull with how it plays, the drawn range, or how many beats and lulls are
// gathered with Shift. A lane's own choice has none here.

/** How long a lull trimmed to a breath keeps, in seconds. */
export const BREATH_S = 0.3;

/** The prefix of a moment's beat id, before its index
 * (`recording-beats.ts`). */
const MOMENT_PREFIX = "moment-";

export function stripIslandItem({
  artifactId,
  beats,
  lulls,
  moments,
  selection,
}: {
  artifactId: number;
  beats: RecordingBeat[];
  lulls: RecordingStripLulls;
  moments: RecordingMoment[];
  selection: StripSelection | null;
}): ToolPanelStripItem | null {
  switch (selection?.kind) {
    case "range":
      return {
        id: `${String(selection.start)}-${String(selection.end)}`,
        kind: "range",
      };
    case "items":
      return {
        count: selection.items.length,
        id: selection.items.map(({ id, kind }) => `${kind}:${id}`).join(","),
        kind: "stripGroup",
      };
    case "lull": {
      const lull = lulls.lulls.find(({ id }) => id === baseOf(selection.id));
      return lull ? { id: selection.id, kind: "lull", play: lull.play } : null;
    }
    case "beat": {
      const found = beats.find(({ id }) => id === baseOf(selection.id));
      if (!found) return null;
      const { beat } = found;
      if (beat.kind !== "moment")
        return { beat, id: selection.id, kind: "beat" };
      const index = Number(found.id.slice(MOMENT_PREFIX.length));
      const note =
        moments.find((moment) => moment.index === index)?.note ?? null;
      return {
        artifactId,
        color: beat.color,
        id: selection.id,
        index,
        kind: "moment",
        name: beat.name,
        note: note && { peaks: note.waveform, transcript: note.transcript },
      };
    }
    default:
      return null;
  }
}
