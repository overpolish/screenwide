// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import {
  GeneralSettings,
  GlideSettings,
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

export const getOcrSettings = () => invoke<OcrSettings>("get_ocr_settings");

export const setOcrSettings = (settings: OcrSettings) =>
  invoke<OcrSettings>("set_ocr_settings", { settings });

export const browseDefaultLocation = (kind: "recording" | "screenshot") =>
  invoke<string | null>("browse_default_location", { kind });
