// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

import {
  AnnotationTool,
  useAnnotationAngleDefault,
  useAnnotationAnimatedDefault,
  useAnnotationDefaults,
  useAnnotationStickerDefault,
} from "../../annotations/annotation-defaults";
import { drawingToolKind } from "../../tool-panels/tool-registry";
import { RecordingVideoTrackId } from "../../types";

import { RecordingAnnotationClip } from "./recording-annotations";

/**
 * Tell the native preview the clips it draws, what is chosen, and what the
 * tool in hand makes next.
 *
 * A fresh annotation's dress and whether it animates both travel with the
 * clips the native tool draws from; animation is the annotation's own
 * property, so it rides beside the style rather than inside it, and so do
 * where a fresh counter's tail points and the picture a fresh sticker shows.
 * The dress is the tool's own: a disc and a stroke are different
 * measurements, so the size the counter tool sends is the one counters were
 * last drawn at.
 */
export function useRecordingAnnotationSync({
  clips,
  selectedIds,
  sessionId,
  tool,
  trackId,
}: {
  clips: RecordingAnnotationClip[];
  selectedIds: ReadonlySet<string>;
  sessionId: number | null;
  tool: AnnotationTool | null;
  trackId: RecordingVideoTrackId | null;
}) {
  const defaults = useAnnotationDefaults(drawingToolKind(tool) ?? "arrow");
  const animated = useAnnotationAnimatedDefault();
  const counterAngle = useAnnotationAngleDefault();
  const sticker = useAnnotationStickerDefault();
  useEffect(() => {
    if (sessionId === null) return;
    void invoke("set_recording_preview_annotations", {
      animated,
      clips,
      counterAngle,
      defaults,
      paneIndex: trackId === null ? null : trackId === "primary" ? 0 : 1,
      selectedIds: [...selectedIds],
      sessionId,
      sticker,
    }).catch((cause: unknown) => {
      console.error("Could not update recording annotations", cause);
    });
  }, [
    animated,
    clips,
    counterAngle,
    defaults,
    selectedIds,
    sessionId,
    sticker,
    trackId,
  ]);
}
