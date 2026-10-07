// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AudioLines,
  Flag,
  Keyboard,
  LayoutGrid,
  PenTool,
  Ruler,
  ScanText,
  Settings,
} from "lucide-react";
import { type ReactNode } from "react";

import { sectionTitles, type SettingsSection } from "./settings-sections";

const icons: [SettingsSection, ReactNode][] = [
  ["general", <Settings key="general" />],
  ["glide", <LayoutGrid key="glide" />],
  ["ruler", <Ruler key="ruler" />],
  ["annotate", <PenTool key="annotate" />],
  ["ocr", <ScanText key="ocr" />],
  ["moments", <Flag key="moments" />],
  ["transcription", <AudioLines key="transcription" />],
  ["hotkeys", <Keyboard key="hotkeys" />],
];

/** The sections the sidebar lists, in its order, each under its title. */
export const settingsSidebarItems = icons.map(([id, icon]) => ({
  icon,
  id,
  label: sectionTitles[id],
}));
