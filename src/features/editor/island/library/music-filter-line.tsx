// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import {
  boundsAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";
import { IslandLineButton } from "../island-line-button";

import {
  MUSIC_FILTER_ORDER,
  MUSIC_FILTERS,
  type MusicFilterName,
  type MusicFilters,
} from "./music-filters";

// Under the Library's search field while it shows music: what songs are
// narrowed by, in the library and online, a pill for each filter showing
// what it is set to, which opens its choices in a menu. Words find a sound
// effect, named for what it is, but not a song, chosen for how it sounds.

/** How wide a filter's menu opens, in px. */
const MENU_PX = 200;

/** The menu's choice that narrows nothing. */
const ANY = "any";

const MENU_PREFIX = "library-music-filter:";

const isFilterName = (name: string): name is MusicFilterName =>
  MUSIC_FILTER_ORDER.some((each) => each === name);

export function MusicFilterLine({
  filters,
  onChange,
}: {
  filters: MusicFilters;
  /** Hands on the filters with one changed. */
  onChange: (filters: MusicFilters) => void;
}) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: t("editor-island-music-filters"),
    mode: "select",
    onSelect: (choice, name) => {
      if (!isFilterName(name)) return;
      onChange({ ...filters, [name]: choice === ANY ? undefined : choice });
    },
    showsAllItems: true,
    width: MENU_PX,
  });
  return (
    <div
      aria-label={t("editor-island-music-filters")}
      className="flex items-center gap-separate"
      role="group"
    >
      {MUSIC_FILTER_ORDER.map((name) => {
        const filter = MUSIC_FILTERS[name];
        const value = filters[name];
        return (
          <IslandLineButton
            aria-haspopup="listbox"
            aria-label={
              value === undefined
                ? filter.name()
                : t("editor-island-music-filter-set", {
                    choice: filter.label(value),
                    filter: filter.name(),
                  })
            }
            className="w-auto shrink-0"
            isChosen={value !== undefined}
            key={name}
            onPress={(event) => {
              void openMenu({
                anchor: boundsAnchor(event.target.getBoundingClientRect()),
                context: name,
                items: [
                  { id: ANY, label: filter.any() },
                  ...filter.choices.map((choice) => ({
                    id: choice,
                    label: filter.label(choice),
                  })),
                ],
                selectedIds: [value ?? ANY],
              });
            }}
          >
            <span className="whitespace-nowrap">
              {value === undefined ? filter.name() : filter.label(value)}
            </span>
            <ChevronDown
              aria-hidden
              className="size-icon-sm shrink-0 text-fg-secondary"
            />
          </IslandLineButton>
        );
      })}
    </div>
  );
}
