// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useLayoutEffect, useRef } from "react";
import {
  Autocomplete,
  ListLayout,
  useFilter,
  Virtualizer,
} from "react-aria-components";

import { SearchField } from "../../components/base/input-fields/search-field";
import { ListBox } from "../../components/base/listbox/listbox";

import { PopupPanelItem, PopupPanelListContent } from "./store";

/** A row's height before it is measured: the compact control height. */
const ESTIMATED_ROW_HEIGHT = 24;

/**
 * A long list of single choices, found by typing: a search field that takes
 * the keys the moment the list opens, the arrows reaching into the rows from
 * it, and only the rows in view drawn, so hundreds of fonts open at once.
 * The window was opened at the size it keeps; the list opens with the
 * current choice in view.
 */
export function PopupPanelSearchList({
  content,
  label,
  onChoose,
  renderItem,
  searchLabel,
}: {
  content: PopupPanelListContent;
  label: string;
  /** A choice made from the keys, which the rows' own press does not see. */
  onChoose: (item: PopupPanelItem) => void;
  renderItem: (item: PopupPanelItem) => ReactNode;
  searchLabel: string;
}) {
  const { contains } = useFilter({ sensitivity: "base" });
  const listRef = useRef<HTMLDivElement>(null);
  const chosen = content.selectedIds[0] as string | undefined;

  // The virtualizer sizes its rows over the frames after mounting, so the
  // list is only tall enough to scroll to the choice two frames in.
  useLayoutEffect(() => {
    const at = content.items.findIndex((item) => item.id === chosen);
    if (at < 0) return;
    let frame = requestAnimationFrame(() => {
      frame = requestAnimationFrame(() => {
        const list = listRef.current;
        if (!list) return;
        const row =
          list.querySelector("[role=option]")?.getBoundingClientRect().height ??
          ESTIMATED_ROW_HEIGHT;
        list.scrollTop = Math.max(0, at * row - list.clientHeight / 2);
      });
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [chosen, content.items]);

  return (
    <div className="window-surface rounded-window windows:inset-ring windows:inset-ring-popover-stroke flex h-dvh flex-col gap-tight overflow-hidden p-tight">
      <Autocomplete filter={contains}>
        <SearchField
          aria-label={searchLabel}
          autoFocus
          placeholder={searchLabel}
        />
        <Virtualizer
          layout={ListLayout}
          layoutOptions={{ estimatedRowHeight: ESTIMATED_ROW_HEIGHT }}
        >
          <ListBox
            aria-label={label}
            // A block, not the list's usual flex column, so the virtualizer's
            // full-height sizer is not shrunk to the window and still scrolls.
            className="block min-h-0 w-full flex-1 overflow-auto"
            items={content.items}
            onSelectionChange={(selection) => {
              if (selection === "all") return;
              const [id] = [...selection];
              const item = content.items.find(
                (candidate) => candidate.id === id,
              );
              if (item) onChoose(item);
            }}
            ref={listRef}
            selectedKeys={content.selectedIds}
            selectionBehavior="toggle"
            selectionMode="single"
            // A choice puts the window away. Made as the button went down,
            // the window was gone before it came up, and WebKit, never seeing
            // the release, dropped the next press when the list came back:
            // the first click there did nothing.
            shouldSelectOnPressUp
          >
            {renderItem}
          </ListBox>
        </Virtualizer>
      </Autocomplete>
    </div>
  );
}
