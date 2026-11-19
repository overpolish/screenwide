// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { invoke } from "@tauri-apps/api/core";

import type { TeleprompterCuePicture } from "../../../bindings/TeleprompterCuePicture";
import type { TeleprompterReadingLine } from "../../../bindings/TeleprompterReadingLine";

export type { TeleprompterCuePicture, TeleprompterReadingLine };

/** What the cue's window shows, or null as the cue goes. */
export const CUE_SHOW_EVENT = "teleprompter-cue://show";
/** Whether the cue stands beside the teleprompter. */
export const CUE_BESIDE_EVENT = "teleprompter://cue-beside";
/** The cue's window was resized by hand to this height, in points. */
export const CUE_HEIGHT_EVENT = "teleprompter://cue-height";

/** Shows `picture` beside the teleprompter, level with its reading `line`,
 * or puts the cue away; false when there is no room beside it. `height` is
 * the one kept from a resize by hand, in points. */
export const showTeleprompterCue = ({
  height = null,
  line = null,
  picture,
}: {
  picture: TeleprompterCuePicture | null;
  height?: number | null;
  line?: TeleprompterReadingLine | null;
}) => invoke<boolean>("show_teleprompter_cue", { height, line, picture });

export const teleprompterCue = () =>
  invoke<TeleprompterCuePicture | null>("teleprompter_cue");

export const hideTeleprompterCue = () => invoke("hide_teleprompter_cue");
