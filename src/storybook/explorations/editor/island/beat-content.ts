// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  beatIsland,
  type BeatWhat,
} from "../../../../features/editor/island/beat/beat-island";
import { momentIsland } from "../../../../features/editor/island/moment/moment-island";
import { type Editor } from "../model/use-editor";
import { type Beat } from "../recording";

import { type IslandContent } from "./island-parts";

/** What happens at one of the exploration's beats, as the beat island says
 * it; null for a stretch of nothing, which is a lull rather than a beat. */
function beatWhat(beat: Beat): BeatWhat | null {
  if (beat.kind === "speech") return { kind: "speech", line: beat.text };
  if (beat.kind === "cue") {
    return beat.cue === "sound" ? { kind: "sound" } : { kind: "screen" };
  }
  const { kind, label } = beat.action;
  switch (kind) {
    case "click":
    case "type":
      return { kind, label };
    // Key labels are written as their symbols run together, such as "⌘S".
    case "keys":
      return { keys: Array.from(label), kind: "keys" };
    case "switch":
      return { app: label, kind: "switch" };
    case "idle":
      return null;
  }
}

/** A beat on the exploration's strip, as the editor's beat island holds
 * it, or the moment island for a flagged moment. The exploration's moments
 * carry no kind or voice note, so each shows as a Notable moment without
 * one. */
export function beatContent(editor: Editor, beat: Beat): IslandContent | null {
  if (beat.kind === "cue" && beat.cue === "moment") {
    const { actions, key, title } = momentIsland({
      color: "#0088ff",
      id: beat.id,
      name: "Notable",
      note: null,
      onCut: editor.remove,
      onDownloadModel: () => undefined,
      onTogglePlay: () => undefined,
    });
    return { actions, from: { kind: "strip" }, key, title };
  }
  const what = beatWhat(beat);
  if (!what) return null;
  const { actions, key, title } = beatIsland({
    beat: what,
    id: beat.id,
    onCut: editor.remove,
  });
  return { actions, from: { kind: "strip" }, key, title };
}
