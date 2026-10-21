// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";
import {
  Header,
  ListBoxSection as AriaListBoxSection,
} from "react-aria-components";

import { t } from "../../i18n/i18n";
import { ListBox } from "../../primitives/listbox/listbox";
import { ScrollArea } from "../../primitives/scroll-area/scroll-area";

import { hidePopupPanel } from "./api";
import { renderPopupPanelItem } from "./popup-panel-list-item";
import { PopupPanelSearchList } from "./popup-panel-search-list";
import {
  PopupPanelItem,
  PopupPanelListContent,
  usePopupPanelStore,
} from "./store";
import { useFittedListWindow } from "./use-fitted-list-window";

type PopupPanelGroup = {
  id: string;
  items: PopupPanelItem[];
  section?: string;
};

/** Runs of consecutive items that name the same section become one group; a
 * run that names none stays a group without a header. */
const groupItemsBySection = (items: PopupPanelItem[]): PopupPanelGroup[] => {
  const groups: PopupPanelGroup[] = [];
  for (const item of items) {
    const last = groups.length > 0 ? groups[groups.length - 1] : null;
    if (last && last.section === item.section) {
      last.items.push(item);
    } else {
      groups.push({
        id: `${groups.length.toString()}:${item.section ?? ""}`,
        items: [item],
        section: item.section,
      });
    }
  }
  return groups;
};

/** The panel as a list of choices: a menu, a pop-up button's list, or a
 * checklist. It fits the window to its items (`useFittedListWindow`). */
export function PopupPanelList({
  content,
  focusContents,
  id: activeId,
  label,
  panel,
}: {
  content: PopupPanelListContent;
  focusContents: boolean;
  id: string;
  label: string;
  /** The window this list is drawn in, so a selection or a dismissal names
   * its own panel rather than whichever was opened last. */
  panel: string;
}) {
  const close = usePopupPanelStore((state) => state.close);
  const select = usePopupPanelStore((state) => state.select);
  const listboxRef = useRef<HTMLDivElement>(null);
  const selectingRef = useRef(false);
  const activeItemCount = content.items.length;
  const showsAllItems = content.showsAllItems === true;

  useFittedListWindow(listboxRef, {
    content: activeId,
    itemCount: activeItemCount,
    showsAllItems,
  });

  // A menu runs actions rather than holding a pick, so nothing is ever drawn
  // as selected and the gutter that would carry the checkmark is dropped.
  const showsSelection = content.mode === "select";

  if (content.items.length === 0) {
    return (
      <div
        className="window-surface rounded-window px-section flex h-full min-h-16 w-full items-center justify-center overflow-hidden text-center text-xs text-muted"
        ref={listboxRef}
      >
        {t("popup-panel-empty")}
      </div>
    );
  }

  const selectItem = (item: PopupPanelItem) => {
    if (selectingRef.current) return;

    // A toggle flips its own tick in the list it belongs to and stays open;
    // the listener reads which item was pressed rather than the tick list.
    if (item.togglesInPlace) {
      const selectedIds = content.selectedIds.includes(item.id)
        ? content.selectedIds.filter((id) => id !== item.id)
        : [...content.selectedIds, item.id];
      select({ id: activeId, panel, pressedId: item.id, selectedIds });
      return;
    }

    selectingRef.current = true;
    select({
      id: activeId,
      panel,
      pressedId: item.id,
      selectedIds: [item.id],
    });
    close(panel);
    void hidePopupPanel(focusContents, panel).finally(() => {
      selectingRef.current = false;
    });
  };

  const onSelectionChange = (selection: "all" | Set<number | string>) => {
    if (selection === "all") return;
    // A single choice is reported by the item's own press, which also fires
    // for the item already chosen; reporting it here as well sent every press
    // twice, which flipped a toggle straight back.
    if (content.selectionMode === "single") return;

    const selectedIds = new Set(
      [...selection].map((selectedId) => selectedId.toString()),
    );
    const exclusiveId = content.exclusiveId;
    const previouslyExclusive = exclusiveId
      ? content.selectedIds.includes(exclusiveId)
      : false;
    if (exclusiveId && selectedIds.has(exclusiveId) && !previouslyExclusive) {
      select({ id: activeId, panel, selectedIds: [exclusiveId] });
      return;
    }
    if (exclusiveId) selectedIds.delete(exclusiveId);
    if (selectedIds.size === 0 && exclusiveId) selectedIds.add(exclusiveId);
    select({
      id: activeId,
      panel,
      selectedIds: content.items
        .map((item) => item.id)
        .filter((itemId) => selectedIds.has(itemId)),
    });
  };

  const renderItem = (item: PopupPanelItem) =>
    renderPopupPanelItem(item, {
      content,
      onPress: selectItem,
      showsSelection,
    });

  if (content.searchLabel)
    return (
      <PopupPanelSearchList
        content={content}
        label={label}
        onChoose={selectItem}
        renderItem={renderItem}
        searchLabel={content.searchLabel}
      />
    );

  const hasSections = content.items.some((item) => item.section !== undefined);

  return (
    <ScrollArea
      inWindowCorner
      key={activeId}
      // The scrollbar keeps out of the rounded corners.
      rootClassName="window-surface rounded-window windows:inset-ring windows:inset-ring-popover-stroke"
    >
      <ListBox
        aria-label={label}
        autoFocus={focusContents}
        className="w-full overflow-visible"
        onSelectionChange={onSelectionChange}
        ref={listboxRef}
        selectedKeys={showsSelection ? content.selectedIds : []}
        selectionBehavior="toggle"
        selectionMode={content.selectionMode}
      >
        {hasSections
          ? groupItemsBySection(content.items).map((group) => (
              <AriaListBoxSection
                className="flex flex-col"
                id={group.id}
                key={group.id}
              >
                {group.section ? (
                  <Header>
                    <span className="text-heading text-fg-secondary px-control-inset py-control block">
                      {group.section}
                    </span>
                  </Header>
                ) : null}
                {group.items.map(renderItem)}
              </AriaListBoxSection>
            ))
          : content.items.map(renderItem)}
      </ListBox>
    </ScrollArea>
  );
}
