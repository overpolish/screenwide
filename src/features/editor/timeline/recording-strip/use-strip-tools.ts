// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { useExportIslandStore } from "../../export/export-island-store";
import { GIF_LONGEST_MS } from "../../export/recording-format";
import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import { type Playhead } from "../scrub-playhead";
import { type StripSelection } from "../strip/strip-selection";
import { type TimeMap } from "../timeline-scale";

// The strip's tools in hand, the blade or the range tool, one at a time:
// from their buttons, or B for the blade and ⇧R for the range tool. B held
// rather than tapped puts the blade back down as it is let go, as video
// editors do. While the Export island makes a GIF the range tool stays in
// hand over the locked strip, and a range is drawn to start from where none
// is: the GIF's longest from the playhead, or up to the end where less is
// left.

type StripTool = "blade" | "range";

const GIF_RANGE_S = GIF_LONGEST_MS / 1000;
/** How long, in ms, B is held before letting it go puts the blade down. */
const HELD_MS = 300;

export function useStripTools({
  map,
  playhead,
  select,
  selection,
}: {
  map: TimeMap;
  playhead: Playhead;
  select: (next: StripSelection | null) => void;
  selection: StripSelection | null;
}) {
  const [tool, setTool] = useState<StripTool | null>(null);
  const isGifRange = useExportIslandStore((state) => state.isGifRange);
  const hasRange = selection?.kind === "range";

  useEffect(() => {
    if (!isGifRange || hasRange) return;
    const from = Math.min(
      playhead.ratio() * map.duration,
      Math.max(map.duration - GIF_RANGE_S, 0),
    );
    select({
      end: map.toRecorded(Math.min(from + GIF_RANGE_S, map.duration)),
      kind: "range",
      start: map.toRecorded(from),
    });
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- drawn once as GIF is chosen, where no range is.
  }, [isGifRange]);

  const toggle = (next: StripTool) => (isOn: boolean) => {
    setTool(isOn ? next : null);
  };
  // The tool in hand before B went down, and when it did, while it is down.
  const bladeKeyRef = useRef<{ at: number; before: StripTool | null } | null>(
    null,
  );
  useEditorWindowShortcuts({
    onToggleBladeTool: () => {
      bladeKeyRef.current ??= { at: performance.now(), before: tool };
      setTool(tool === "blade" ? null : "blade");
    },
    onToggleRangeTool: () => {
      setTool(tool === "range" ? null : "range");
    },
  });
  useEffect(() => {
    const onKeyUp = (event: KeyboardEvent) => {
      const down = bladeKeyRef.current;
      if (event.code !== "KeyB" || !down) return;
      bladeKeyRef.current = null;
      if (performance.now() - down.at >= HELD_MS) setTool(down.before);
    };
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keyup", onKeyUp);
    };
  }, []);
  const isCutting = tool === "blade";
  const isRanging = tool === "range" || isGifRange;
  return {
    strip: {
      gifRangeS: isGifRange ? GIF_RANGE_S : undefined,
      isCutting,
      isRanging,
    },
    transport: {
      isCutting,
      isGifRange,
      isRanging,
      onCuttingChange: toggle("blade"),
      onRangingChange: toggle("range"),
    },
  };
}
