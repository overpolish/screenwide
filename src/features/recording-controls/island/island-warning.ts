// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import { INPUTS } from "./island-names";

import type {
  IslandInput,
  IslandInputKind,
  RecordingIslandProps,
} from "./island-types";

// What, if anything, the resting island must say before recording can go
// ahead, and the one action that fixes it. Permission comes first, since
// nothing records without it; then a switched-on input that cannot record.
// With no screen and no input, the No Screen part itself says what to do,
// and Record waits.

export type IslandWarning = {
  message: string;
  action?: string;
  onAction?: () => void;
};

/** An input that is on and could be recorded: allowed, and with nothing
 * wrong with it. */
const isUsable = (input: IslandInput) =>
  input.isOn && !input.isLocked && !input.warning;

/** Whether anything at all would be recorded: the screen, unless it is left
 * out, or else at least one input that can be. A screen recording goes ahead
 * without a broken input, as the recording bar's did; with no screen, that
 * input may be all there is. */
const hasSomethingToRecord = (props: RecordingIslandProps) =>
  props.mode !== "none" || INPUTS.some((kind) => isUsable(props[kind]));

/** Whether the screen and the part of it the mode records are ready: allowed,
 * and in the Window and Region modes the window or area chosen. */
const hasScreen = (props: RecordingIslandProps) =>
  props.mode !== "none" &&
  !props.isScreenLocked &&
  !(props.mode === "window" && !props.chosenWindow) &&
  !(props.mode === "region" && !props.chosenRegion);

/** Whether Record can start: the screen's permission when it is recorded,
 * something to record, and in the Window and Region modes the window or area
 * chosen. */
export const canRecord = (props: RecordingIslandProps) =>
  hasSomethingToRecord(props) && (props.mode === "none" || hasScreen(props));

/** Whether a screenshot can be taken: there is a screen ready to capture and
 * no capture already under way. */
export const canScreenshot = (props: RecordingIslandProps) =>
  hasScreen(props) && props.screenshotState !== "pending";

export function islandWarning(
  props: RecordingIslandProps,
  openInput: (kind: IslandInputKind) => void,
): IslandWarning | undefined {
  const permissions = {
    action: t("recording-controls-open-permissions"),
    onAction: props.onOpenPermissions,
  };
  if (props.mode !== "none" && props.isScreenLocked) {
    return {
      ...permissions,
      message: t("recording-controls-island-screen-locked"),
    };
  }
  if (props.camera.isOn && props.camera.isLocked) {
    return {
      ...permissions,
      message: t("recording-controls-island-camera-locked"),
    };
  }
  if (props.microphone.isOn && props.microphone.isLocked) {
    return {
      ...permissions,
      message: t("recording-controls-island-microphone-locked"),
    };
  }
  const broken = INPUTS.find((kind) => props[kind].isOn && props[kind].warning);
  if (!broken) return undefined;
  return {
    action: t("recording-controls-island-choose-another"),
    message: props[broken].warning ?? "",
    onAction: () => {
      openInput(broken);
    },
  };
}
