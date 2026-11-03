// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { TextRecognitionIslandAction } from "../../bindings/TextRecognitionIslandAction";
import type { TextRecognitionIslandState } from "../../bindings/TextRecognitionIslandState";

export const cancelTextRecognition = () =>
  invoke<null>("cancel_text_recognition");

export const startTextRecognition = () =>
  invoke<null>("start_text_recognition");

export const copyRecognitionContent = (text: string) =>
  invoke<null>("copy_recognition_content", { text });

export const getTextRecognitionIslandState = () =>
  invoke<TextRecognitionIslandState | null>("text_recognition_island_state");

export const runTextRecognitionIslandAction = (
  action: TextRecognitionIslandAction,
) => invoke<null>("text_recognition_island_action", { action });
