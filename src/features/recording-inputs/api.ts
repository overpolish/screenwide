// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

export type RecordingOptionsState = {
  focusContents: boolean;
  open: boolean;
  revision: number;
};

export type RecordingOptionsAnchor = {
  height: number;
  width: number;
  x: number;
  y: number;
};

export const toggleRecordingOptions = (
  anchor: RecordingOptionsAnchor,
  focusContents: boolean,
) => invoke<null>("toggle_recording_options", { anchor, focusContents });

export const hideRecordingOptions = () =>
  invoke<null>("hide_recording_options");

export const getRecordingOptionsState = () =>
  invoke<RecordingOptionsState>("get_recording_options_state");

export const setRecordingOptionsContentHeight = (height: number) =>
  invoke<null>("set_recording_options_content_height", { height });
