// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { useBuiltInBackgroundPresets } from "../../tools/frame/use-background-presets";
import { useToolPanelSnapshot } from "../../tools/use-tool-panel-snapshot";
import { EditorKind } from "../../types";
import { type EditorIslandContent } from "../editor-island";
import { type SceneToolGroup } from "../scene/scene-actions";

import { annotationIslandContent } from "./annotation-island-content";
import { captionsIslandContent } from "./captions-island-content";
import { type EditorIslandKind } from "./editor-island-kind";
import { exportIslandContent } from "./export-island-content";
import { frameIslandContent } from "./frame-island-content";
import {
  cameraIslandContent,
  cropIslandContent,
  screenIslandContent,
} from "./layer-island-content";
import { libraryIslandContent } from "./library-island-content";
import { modelWorkIslandContent } from "./model-work-island-content";
import {
  cursorIslandContent,
  keyboardIslandContent,
  soundIslandContent,
  tracksIslandContent,
} from "./recording-island-content";
import { sceneIslandContent } from "./scene-island-content";
import { stripIslandContent } from "./strip-island-content";
import { useAudioClipPreview } from "./use-audio-clip-preview";
import { useMomentNote } from "./use-moment-note";
import { useScriptIsland } from "./use-script-island";

// What the islands hold, made from what the editor publishes to its panel
// window (`ToolPanelSnapshot`) and changed by the asks the islands send it
// back (`ToolPanelPatch`), so the editor has one way to be read or changed.
// While the Export island stands over the editor, its tools are out of
// reach, and every other island is put away. Each adapter lives beside its
// island's kind in this folder.

/** The islands shown at once: the one in the picture's top end corner (or
 * over a strip item), the Script island in its top start corner, for the
 * Captions and Transcript modes, and the models' work in its bottom start
 * corner while there is any. */
export type SnapshotIslands = {
  corner: EditorIslandContent | null;
  script: EditorIslandContent | null;
  work: EditorIslandContent | null;
};

export function useSnapshotIsland(
  island: EditorIslandKind | null,
  workspace: EditorKind,
): SnapshotIslands {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const presets = useBuiltInBackgroundPresets();
  // The Scene island's open group, which opens again for the next scene.
  const [sceneGroup, setSceneGroup] = useState<SceneToolGroup>("layout");
  const script = useScriptIsland({ change, island, snapshot, workspace });
  const moment = useMomentNote(snapshot.stripItem);
  const audioClip = useAudioClipPreview(snapshot.stripItem);
  // The Export island is what locks the editor, so it alone stays out. Out
  // or done, it holds the island's place itself (`useEditorIslandWindow`);
  // tucked away while it exports, every other island stays away too. That
  // is read from the export under way, not the lock: the lock lifts in the
  // next snapshot, a moment after the island it was standing in for comes
  // back, which would put that island away for the moment and bring it out
  // again, rather than reshaping the Export island into it.
  const inputs = { change, snapshot };
  // The models' work goes on whatever is in hand, so its island stays out
  // until there is none; the Export island, out or exporting, puts it away
  // with the rest.
  const work =
    island === "export" || snapshot.export?.progress
      ? null
      : modelWorkIslandContent(inputs);
  if (island === null || (island !== "export" && snapshot.export?.progress))
    return { corner: null, script: null, work };
  const capture = workspace === "screenshot" ? "screenshot" : "recording";
  const corner = (): EditorIslandContent | null => {
    switch (island) {
      case "audioClip":
      case "beat":
      case "gap":
      case "lull":
      case "moment":
      case "range":
      case "stripGroup":
        return stripIslandContent({
          audioClip,
          backgrounds: { presets, snapshot },
          change,
          island,
          item: snapshot.stripItem,
          moment,
        });
      case "annotation":
        return annotationIslandContent({ ...inputs, workspace });
      case "camera":
        return cameraIslandContent(inputs);
      case "captions":
        return script.transcript
          ? captionsIslandContent({ ...inputs, transcript: script.transcript })
          : null;
      case "crop":
        return cropIslandContent(inputs);
      case "cursor":
        return cursorIslandContent(inputs);
      case "export":
        return exportIslandContent({ ...inputs, workspace });
      case "frame":
        return frameIslandContent({ ...inputs, capture, presets });
      case "keyboard":
        return keyboardIslandContent(inputs);
      case "library":
        return libraryIslandContent({ ...inputs, workspace });
      case "scene":
        return sceneIslandContent({
          ...inputs,
          group: sceneGroup,
          onGroupChange: setSceneGroup,
        });
      case "screen":
        return screenIslandContent({ ...inputs, capture });
      case "sound":
        return soundIslandContent(inputs);
      case "tracks":
        return tracksIslandContent(inputs);
      case "transcript":
        return null;
    }
  };
  const shown = corner();
  // The Captions mode's pair comes out together, each in its place, rather
  // than one standing alone until the other is ready.
  if (island === "captions" && (shown === null || script.script === null))
    return { corner: null, script: null, work };
  return { corner: shown, script: script.script, work };
}
