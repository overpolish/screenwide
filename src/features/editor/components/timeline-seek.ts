// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { RecordingAnnotationClip } from "../recording-annotations";
export type ScrubPhase = "end" | "move" | "start";
export type SeekHandler = (
  position: number,
  phase: ScrubPhase,
  annotationClips?: RecordingAnnotationClip[],
) => void;
export const seekSelectionVisible = (phase: ScrubPhase) =>
  phase === "move" ? undefined : phase === "end";
