// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { invoke } from "@tauri-apps/api/core";

import type { Script } from "../../bindings/Script";
import type { TeleprompterVoice } from "../../bindings/TeleprompterVoice";
import type { TeleprompterVoiceProblem } from "../../bindings/TeleprompterVoiceProblem";

export type { Script, TeleprompterVoice, TeleprompterVoiceProblem };

/** Another script for the teleprompter to show, by its project's manifest. */
export const TELEPROMPTER_SCRIPT_EVENT = "teleprompter://script";
/** The script shown renamed, by its project's new manifest. */
export const TELEPROMPTER_MOVED_EVENT = "teleprompter://moved";
/** What the voice the teleprompter follows is heard to say. */
export const TELEPROMPTER_VOICE_EVENT = "teleprompter://voice";

/** Makes an empty script and opens the teleprompter on it. */
export const newScript = () => invoke("new_script");

export const readScript = (project: string) =>
  invoke<Script>("read_script", { project });

export const saveScript = (project: string, text: string) =>
  invoke("save_script", { project, text });

/** The script the teleprompter shows, by its project's manifest. */
export const teleprompterScript = () =>
  invoke<string | null>("teleprompter_script");

/** Fills the display the teleprompter is on, or puts it back. */
export const setTeleprompterFullscreen = (fullscreen: boolean) =>
  invoke("set_teleprompter_fullscreen", { fullscreen });

/** Starts following the voice; rejects with why it cannot be. */
export const startTeleprompterVoice = () => invoke("start_teleprompter_voice");

export const stopTeleprompterVoice = () => invoke("stop_teleprompter_voice");
