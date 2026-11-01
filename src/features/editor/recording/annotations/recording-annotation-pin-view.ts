// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { pinCorrections } from "./recording-annotation-pins";
import { RecordingAnnotationPin } from "./recording-annotations";
import { RecordingPinStatus } from "./use-recording-pin-status";

import type {
  AnnotationPinUnsure,
  AnnotationPinView,
} from "../../island/annotation/annotation-pin";

/**
 * Why the tracker is unsure of a pinned annotation's content at `ms`, in
 * source time, or null where it is sure or has not said. Lost comes first:
 * the content was not found at all, where off the frame and under a cover
 * say where it went.
 */
export const pinUnsureAt = (
  status: RecordingPinStatus | undefined,
  ms: number,
): AnnotationPinUnsure | null => {
  if (!status || status.progress !== null) return null;
  const kinds = [
    ["lost", status.weak],
    ["off-frame", status.hidden],
    ["under-cover", status.under],
  ] as const;
  return (
    kinds.find(([, stretches]) =>
      stretches.some(([start, end]) => ms >= start && ms < end),
    )?.[0] ?? null
  );
};

/**
 * How a pinned annotation stands at the playhead, `positionMs` in source
 * time, for its island. Its path is being worked out until the tracker says
 * it has landed, which a fresh pin has not yet said at all; once landed, it
 * follows its content except where it is hidden at the playhead, by the
 * tracker or by the hand (`canShow`, the menu's Content Back in View), and
 * says where the tracker is unsure of it. The answer offered is the menu's
 * for the playhead; where it offers none, the playhead is outside the clip
 * or on its pinned frame, where nothing is unsure to put right.
 */
export const recordingPinView = ({
  canHide,
  canShow,
  pin,
  positionMs,
  status,
}: {
  canHide: boolean;
  canShow: boolean;
  pin: RecordingAnnotationPin;
  positionMs: number;
  status: RecordingPinStatus | undefined;
}): AnnotationPinView => ({
  action: canShow ? "show" : canHide ? "hide" : null,
  hasCorrections: pinCorrections(pin).length > 0,
  progress: status?.progress ?? null,
  state:
    status === undefined || status.progress !== null
      ? "tracking"
      : canShow
        ? "out-of-view"
        : "following",
  unsure: canHide || canShow ? pinUnsureAt(status, positionMs) : null,
});
