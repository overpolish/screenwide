// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

export type CaptionPanelTab = "layout" | "style" | "words";

const TABS: readonly CaptionPanelTab[] = ["layout", "style", "words"];

const KEY = "screenwide.captions.tab";

/** Keeps `tab` as the one the Captions panel opens on. Storage that cannot
 * be written only costs the memory: the panel then opens on Words. */
export const rememberCaptionPanelTab = (tab: CaptionPanelTab) => {
  try {
    localStorage.setItem(KEY, tab);
  } catch {
    // Nothing to do: forgetting the tab is harmless.
  }
};

/** The Captions panel's open tab, kept in the panel window's storage so the
 * panel reopens on it, after a restart too. */
export function useCaptionPanelTab() {
  const [tab, setTab] = useState<CaptionPanelTab>(() => {
    try {
      const kept = localStorage.getItem(KEY);
      return TABS.find((each) => each === kept) ?? "words";
    } catch {
      return "words";
    }
  });
  const choose = (next: CaptionPanelTab) => {
    setTab(next);
    rememberCaptionPanelTab(next);
  };
  return [tab, choose] as const;
}
