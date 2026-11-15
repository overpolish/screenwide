// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { DictationIslandAction } from "../../bindings/DictationIslandAction";
import type { DictationIslandState } from "../../bindings/DictationIslandState";

export const getDictationIslandState = () =>
  invoke<DictationIslandState | null>("dictation_island_state");

export const runDictationIslandAction = (action: DictationIslandAction) =>
  invoke<null>("dictation_island_action", { action });

/** The recording island's microphone, by id, which dictation listens to;
 * null for the system's default. */
export const setDictationMicrophone = (microphone: string | null) =>
  invoke<null>("set_dictation_microphone", { microphone });
