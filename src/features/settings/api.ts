// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import {
  getPermissionSnapshot,
  openPermissionSettings,
  requestPermission,
} from "../permissions/api";
import { listMicrophones } from "../recording-inputs/devices-api";

import {
  AnnotateSettings,
  GeneralSettings,
  GlideSettings,
  MomentSettings,
  OcrSettings,
  RulerSettings,
  ShortcutAction,
  ShortcutSettings,
  ShortcutDefaults,
} from "./types";

export const getShortcutDefaults = () =>
  invoke<ShortcutDefaults>("get_shortcut_defaults");

export const getShortcutSettings = () =>
  invoke<ShortcutSettings>("get_shortcut_settings");

export const setShortcutBinding = (
  action: ShortcutAction,
  shortcut: string | null,
) => invoke<ShortcutSettings>("set_shortcut_binding", { action, shortcut });

export const beginShortcutCapture = () =>
  invoke<null>("begin_shortcut_capture");

export const endShortcutCapture = () => invoke<null>("end_shortcut_capture");

export const hideSettings = () => invoke<null>("hide_settings");

export const getGeneralSettings = () =>
  invoke<GeneralSettings>("get_general_settings");

export const setGeneralSettings = (settings: GeneralSettings) =>
  invoke<GeneralSettings>("set_general_settings", { settings });

export const getGlideSettings = () =>
  invoke<GlideSettings>("get_glide_settings");

export const setGlideSettings = (settings: GlideSettings) =>
  invoke<GlideSettings>("set_glide_settings", { settings });

export const getRulerSettings = () =>
  invoke<RulerSettings>("get_ruler_settings");

export const setRulerSettings = (settings: RulerSettings) =>
  invoke<RulerSettings>("set_ruler_settings", { settings });

export const getAnnotateSettings = () =>
  invoke<AnnotateSettings>("get_annotate_settings");

export const setAnnotateSettings = (settings: AnnotateSettings) =>
  invoke<AnnotateSettings>("set_annotate_settings", { settings });

/** Fired on every write, so the toolbar and the Settings page show one state
 * without either asking the other - and neither writes back a copy the other
 * has since changed. */
const ANNOTATE_CHANGED_EVENT = "annotate-settings://changed";

export const listenToAnnotateSettings = (
  onChange: (settings: AnnotateSettings) => void,
) =>
  listen<AnnotateSettings>(ANNOTATE_CHANGED_EVENT, (event) => {
    onChange(event.payload);
  });

export const getOcrSettings = () => invoke<OcrSettings>("get_ocr_settings");

export const setOcrSettings = (settings: OcrSettings) =>
  invoke<OcrSettings>("set_ocr_settings", { settings });

export const getMomentSettings = () =>
  invoke<MomentSettings>("get_moment_settings");

export const setMomentSettings = (settings: MomentSettings) =>
  invoke<MomentSettings>("set_moment_settings", { settings });

/** Whether the app may use a microphone, as voice notes need. */
export const getMicrophoneAccess = () =>
  getPermissionSnapshot().then((snapshot) => snapshot.microphone);

export const requestMicrophoneAccess = () => requestPermission("microphone");

export const openMicrophoneSettings = () =>
  openPermissionSettings("microphone");

export { listMicrophones };

export {
  cancelTranscriptionDownload,
  downloadTranscriptionModel,
  getTranscriptionState,
  listenToTranscription,
  removeTranscriptionModel,
  setTranscriptionLanguage,
} from "../transcription/api";

export const browseDefaultLocation = (
  kind: "project" | "recording" | "screenshot",
) => invoke<string | null>("browse_default_location", { kind });
