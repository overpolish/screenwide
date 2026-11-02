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
  momentNames,
  setSection,
}: {
  models: readonly TranscriptionModel[];
  /** The names of the person's moments, whose shortcuts it finds. */
  momentNames: readonly string[];
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
    () => settingsSearchEntries({ isMac: isMacHost, models, momentNames }),
    [models, momentNames],
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
      if (pane && revealSetting(pane, opened.entry)) {
        // Revealed, it is done with: a pane opened later starts as usual.
        setOpened(null);
        return;
      }
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

  const open = (entry: SettingsSearchEntry) => {
    setQuery("");
    setSection(entry.section);
    setOpened({ entry });
  };

  return {
    isActive: query.trim() !== "",
    open,
    /** Opens the row of `title` in `section`, as a result would. */
    openRow: (section: SettingsSection, title: string) => {
      const entry = entries.find(
        (candidate) =>
          candidate.section === section &&
          candidate.target === "row" &&
          candidate.title === title,
      );
      if (entry) open(entry);
      else setSection(section);
    },
    paneRef,
    query,
    results,
    resultsRef,
    /** The title of the row being brought into view, until it is. */
    revealing: opened?.entry.title ?? null,
    setQuery,
    suggestions,
  };
}

export type SettingsSearch = ReturnType<typeof useSettingsSearch>;
