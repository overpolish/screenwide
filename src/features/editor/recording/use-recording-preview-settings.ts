// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dispatch, RefObject, SetStateAction, useEffect } from "react";

import {
  selectRecordingPreviewAudio,
  setRecordingPreviewAudioVolumes,
  setRecordingPreviewCursorEffects,
  setRecordingPreviewKeyboardEffects,
} from "../api";
import {
  RecordingPreviewKeyboardDeletions,
  setRecordingPreviewDeletedKeyboardShortcuts,
} from "../timeline/keyboard/recording-keyboard-timeline-api";
import {
  AudioTrackVolume,
  CursorEffectSettings,
  KeyboardEffectSettings,
} from "../types";

/**
 * The preview settings React still pushes on their own channel.
 *
 * The composition (camera overlay, recording output, bake) is not among them:
 * the native surface receives it inside every layout invoke, atomically with
 * the pane rects it belongs to and ordered by requestId. Sending it here as
 * well would create a second, unordered channel.
 */
export function useRecordingPreviewSettings({
  audioTrackVolumes,
  cursorEffects,
  enabledStreamIndices,
  isEnabled,
  keyboardDeletions,
  keyboardEffects,
  sessionIdRef,
  setError,
  startedRef,
}: {
  audioTrackVolumes: AudioTrackVolume[];
  cursorEffects: CursorEffectSettings;
  /** The audio tracks turned on, which the player mixes as it plays. */
  enabledStreamIndices: number[];
  isEnabled: boolean;
  keyboardDeletions: RecordingPreviewKeyboardDeletions;
  keyboardEffects: KeyboardEffectSettings;
  sessionIdRef: RefObject<number>;
  setError: Dispatch<SetStateAction<string | null>>;
  startedRef: RefObject<boolean>;
}) {
  const enabledAudio = enabledStreamIndices.join("-");
  const volumes = audioTrackVolumes
    .map(
      ({ decibels, streamIndex }) =>
        `${streamIndex.toString()}:${decibels.toString()}`,
    )
    .join("-");
  const cursor = Object.values(cursorEffects).join("-");
  const deletedKeyboardShortcuts = JSON.stringify(keyboardDeletions);
  const keyboard = Object.values(keyboardEffects).join("-");
  useEffect(() => {
    if (!isEnabled || !startedRef.current) return;
    void setRecordingPreviewAudioVolumes(
      audioTrackVolumes,
      sessionIdRef.current,
    ).catch(setError);
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [isEnabled, volumes]);
  useEffect(() => {
    if (!isEnabled || !startedRef.current) return;
    void selectRecordingPreviewAudio(
      enabledStreamIndices,
      sessionIdRef.current,
    ).catch(setError);
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [enabledAudio, isEnabled]);
  useEffect(() => {
    if (!isEnabled || !startedRef.current) return;
    void setRecordingPreviewCursorEffects(
      cursorEffects,
      sessionIdRef.current,
    ).catch(setError);
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [cursor, isEnabled]);
  useEffect(() => {
    if (!isEnabled || !startedRef.current) return;
    void setRecordingPreviewKeyboardEffects(
      keyboardEffects,
      sessionIdRef.current,
    ).catch(setError);
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [isEnabled, keyboard]);
  useEffect(() => {
    if (!isEnabled || !startedRef.current) return;
    void setRecordingPreviewDeletedKeyboardShortcuts(
      keyboardDeletions,
      sessionIdRef.current,
    ).catch(setError);
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [deletedKeyboardShortcuts, isEnabled]);
}
