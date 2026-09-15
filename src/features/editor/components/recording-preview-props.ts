// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
} from "../recording-export-settings";

import type { ScrubPreviewProps } from "./scrub-preview";

const EMPTY_AUDIO_TRACKS: NonNullable<ScrubPreviewProps["audioTracks"]> = [];

/** The preview's props once every optional one has been given its default, so
 * the wiring below reads one shape rather than repeating the fallbacks. */
export type ResolvedScrubPreviewProps = ScrubPreviewProps &
  Required<
    Pick<
      ScrubPreviewProps,
      | "audioTrackVolumes"
      | "audioTracks"
      | "bakeCamera"
      | "cameraOverlay"
      | "cursorEffects"
      | "enabledVideoTracks"
      | "hasCursorData"
      | "hasKeyboardData"
      | "isExportOpen"
      | "isPreparingAudio"
      | "isPreparingPreview"
      | "isSaving"
      | "keyboardEffects"
      | "selectedTrack"
    >
  >;

/** Fills the preview's defaults in exactly as the parameter list once did: an
 * omitted prop is defaulted on every render, a given one keeps its identity. */
export function resolveScrubPreviewProps(
  props: ScrubPreviewProps,
): ResolvedScrubPreviewProps {
  const {
    audioTrackVolumes = [],
    audioTracks = EMPTY_AUDIO_TRACKS,
    bakeCamera = false,
    cameraOverlay = defaultCameraOverlay(),
    cursorEffects = {
      bake: true,
      clickAnimation: true,
      clipAtVideoEdge: false,
      motionBlur: true,
      sizePercent: 100,
      smoothMovement: true,
    },
    enabledVideoTracks = [],
    hasCursorData = false,
    hasKeyboardData = false,
    isExportOpen = false,
    isPreparingAudio = false,
    isPreparingPreview = false,
    isSaving = false,
    keyboardEffects = DEFAULT_KEYBOARD_EFFECTS,
    selectedTrack = null,
  } = props;
  return {
    ...props,
    audioTrackVolumes,
    audioTracks,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    enabledVideoTracks,
    hasCursorData,
    hasKeyboardData,
    isExportOpen,
    isPreparingAudio,
    isPreparingPreview,
    isSaving,
    keyboardEffects,
    selectedTrack,
  };
}
