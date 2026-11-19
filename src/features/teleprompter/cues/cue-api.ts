// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { invoke } from "@tauri-apps/api/core";

import type { ShownTeleprompterCue } from "../../../bindings/ShownTeleprompterCue";
import type { TeleprompterCue } from "../../../bindings/TeleprompterCue";
import type { TeleprompterCueTiming } from "../../../bindings/TeleprompterCueTiming";

export type { ShownTeleprompterCue, TeleprompterCue };

/** A cue made for the teleprompter's script, to pin and keep. */
export const TELEPROMPTER_CUE_EVENT = "teleprompter://cue";

export const readCues = (project: string) =>
  invoke<ShownTeleprompterCue[]>("read_cues", { project });

export const saveCues = (project: string, cues: TeleprompterCue[]) =>
  invoke("save_cues", { cues, project });

/** The moment `sourceMs` into a recording as its edit shows it, small, as
 * a bitmap: opaque to React, whose development build walks the props it
 * logs and took seconds over a frame's pixels. */
export const cuePreview = async (source: string, sourceMs: number) => {
  const bytes = await invoke<ArrayBuffer>("cue_preview", {
    source,
    sourceMs: Math.round(sourceMs),
  });
  // Its width and height, then its pixels, as `cue_preview` lays them out.
  const size = new DataView(bytes, 0, 8);
  const width = size.getUint32(0, true);
  const height = size.getUint32(4, true);
  return createImageBitmap(
    new ImageData(new Uint8ClampedArray(bytes, 8), width, height),
  );
};

/** Makes a cue of `source` pinned to the word starting `at` in `script`: a
 * recording's at `timing`, a screenshot's without. */
export const addCue = ({
  at,
  script,
  source,
  timing,
}: {
  at: number;
  script: string;
  source: string;
  timing: TeleprompterCueTiming | null;
}) =>
  invoke("add_cue", {
    at,
    script,
    source,
    timing: timing && {
      playedMs: Math.round(timing.playedMs),
      sourceMs: Math.round(timing.sourceMs),
    },
  });
