// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useRef, useState } from "react";

import { isMacHost } from "../../../lib/platform";

import { matchSettings } from "./match-settings";
import { revealSetting } from "./reveal-setting";
import {
  settingsSearchEntries,
  type SettingsSearchEntry,
} from "./settings-search-index";

import type { TranscriptionModel } from "../../transcription/types";
import type { SettingsSection } from "../settings-sections";

// A pane draws its rows once its settings load, so an opened result looks
// for its row for a few frames before giving up.
const REVEAL_FRAMES = 30;

export function useSettingsSearch({
  initialQuery = "",
  models,
  setSection,
}: {
  models: readonly TranscriptionModel[];
  setSection: (section: SettingsSection) => void;
  initialQuery?: string;
}) {
  const [query, setQuery] = useState(initialQuery);
  // A fresh object each time, so opening the same result again reveals it
  // again.
  const [opened, setOpened] = useState<{ entry: SettingsSearchEntry } | null>(
    null,
  );
  const paneRef = useRef<HTMLElement>(null);
  const resultsRef = useRef<HTMLDivElement>(null);
  const entries = useMemo(
    () => settingsSearchEntries({ isMac: isMacHost, models }),
    [models],
  );
  const results = useMemo(
    () => matchSettings(entries, query),
    [entries, query],
  );
  // The first row of each pane, offered when nothing matches.
  const suggestions = useMemo(
    () =>
      entries.filter(
        (entry, index) =>
          entries.findIndex(({ section }) => section === entry.section) ===
          index,
      ),
    [entries],
  );

  useEffect(() => {
    if (!opened) return;
    let frames = 0;
    let frame = requestAnimationFrame(function look() {
      const pane = paneRef.current;
      if (pane && revealSetting(pane, opened.entry)) return;
      frames += 1;
      if (frames < REVEAL_FRAMES) frame = requestAnimationFrame(look);
      else if (import.meta.env.DEV)
        console.error(
          `Settings search found no row for "${opened.entry.title}".`,
        );
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [opened]);

  return {
    isActive: query.trim() !== "",
    open: (entry: SettingsSearchEntry) => {
      setQuery("");
      setSection(entry.section);
      setOpened({ entry });
    },
    paneRef,
    query,
    results,
    resultsRef,
    setQuery,
    suggestions,
  };
}

export type SettingsSearch = ReturnType<typeof useSettingsSearch>;
