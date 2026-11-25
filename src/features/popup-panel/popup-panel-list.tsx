// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef } from "react";
import {
  Header,
  ListBoxSection as AriaListBoxSection,
} from "react-aria-components";

import { t } from "../../i18n/i18n";
import { ListBox } from "../../primitives/listbox/listbox";
import { revealChosen } from "../../primitives/scroll-area/reveal-chosen";
import { ScrollArea } from "../../primitives/scroll-area/scroll-area";

import { hidePopupPanel } from "./api";
import { scrollingSectionHeight } from "./layout";
import { renderPopupPanelItem } from "./popup-panel-list-item";
import { PopupPanelSearchList } from "./popup-panel-search-list";
import {
  PopupPanelItem,
  PopupPanelListContent,
  usePopupPanelStore,
} from "./store";
import { useFittedListWindow } from "./use-fitted-list-window";
import { useToggleDrag } from "./use-toggle-drag";

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
  const fixedRef = useRef<HTMLDivElement>(null);
  const headerRef = useRef<HTMLDivElement>(null);
  const scrollingRef = useRef<HTMLDivElement>(null);
  const selectingRef = useRef(false);
  const activeItemCount = content.items.length;
  const showsAllItems = content.showsAllItems === true;
  const scrollingSection = content.scrollingSection;
  // The commands above a scrolling section show whole; the section takes
  // what room the list ceiling leaves it (`scrollingSectionHeight`).
  const measureSplit = useCallback(() => {
    const fixed =
      (fixedRef.current?.offsetHeight ?? 0) +
      (headerRef.current?.offsetHeight ?? 0);
    return (
      fixed +
      scrollingSectionHeight(fixed, scrollingRef.current?.offsetHeight ?? 0)
    );
  }, []);

  useFittedListWindow(listboxRef, {
    content: activeId,
    itemCount: activeItemCount,
    ...(scrollingSection === undefined ? {} : { measure: measureSplit }),
    opensAbove: content.opensAbove === true,
    showsAllItems,
  });
  const toggles = useToggleDrag({
    items: content.items,
    onFlip: (item, selectedIds) => {
      select({ id: activeId, panel, pressedId: item.id, selectedIds });
    },
    selectedIds: content.selectedIds,
  });

  // A menu runs actions rather than holding a pick, so nothing is drawn as
  // selected and the gutter that would carry the checkmark is dropped,
  // unless it holds toggles, whose ticks show what is on.
  const showsSelection =
    content.mode === "select" ||
    content.items.some((item) => item.togglesInPlace);

  if (content.items.length === 0) {
    return (
      <div
        className="flex h-full min-h-16 w-full items-center justify-center overflow-hidden px-inset text-center text-caption text-fg-secondary"
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
    // One a drag began on has flipped already (`useToggleDrag`).
    if (item.togglesInPlace) {
      toggles.flipPressed(item);
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
  const renderGroups = (items: PopupPanelItem[]) =>
    groupItemsBySection(items).map((group) => (
      <AriaListBoxSection
        className="flex flex-col"
        id={group.id}
        key={group.id}
      >
        {group.section ? (
          <Header>
            <span className="block px-inset-sm pt-separate pb-inline text-caption text-fg-secondary">
              {group.section}
            </span>
          </Header>
        ) : null}
        {group.items.map(renderItem)}
      </AriaListBoxSection>
    ));
  const listProps = {
    onSelectionChange,
    selectedKeys: showsSelection ? content.selectedIds : [],
    selectionBehavior: "toggle" as const,
    selectionMode: content.selectionMode,
  };

  if (scrollingSection !== undefined) {
    const fixedItems = content.items.filter(
      (item) => item.section !== scrollingSection,
    );
    const scrollingItems = content.items.filter(
      (item) => item.section === scrollingSection,
    );
    // Two lists, the commands and the ticks that scroll under them, so the
    // commands never scroll out of view. A toggle drag runs across both.
    return (
      <div
        className="flex h-full flex-col"
        // Each menu opens with its ticks scrolled to the top, not where the
        // last menu's were left.
        key={activeId}
        onPointerDownCapture={toggles.onPointerDown}
        ref={listboxRef}
        role="presentation"
      >
        <ListBox
          aria-label={label}
          autoFocus={focusContents}
          className="w-full shrink-0 overflow-visible pb-0"
          ref={fixedRef}
          {...listProps}
        >
          {renderGroups(fixedItems)}
        </ListBox>
        <div aria-hidden className="shrink-0 px-inset-sm" ref={headerRef}>
          <span className="block px-inset-sm pt-separate pb-inline text-caption text-fg-secondary">
            {scrollingSection}
          </span>
        </div>
        <ScrollArea inWindowCorner rootClassName="min-h-0 flex-1">
          <ListBox
            aria-label={scrollingSection}
            className="w-full overflow-visible pt-0"
            ref={scrollingRef}
            {...listProps}
          >
            {scrollingItems.map(renderItem)}
          </ListBox>
        </ScrollArea>
      </div>
    );
  }

  return (
    <ScrollArea
      inWindowCorner
      key={activeId}
      // A list that holds a choice opens on it rather than at its top, the
      // first one chosen where several sections each hold one.
      {...(showsSelection ? { onInitialized: revealChosen } : {})}
      // The scrollbar keeps out of the host's rounded corners.
      rootClassName="rounded-panel"
    >
      {/* A press on a toggle starts a drag that sets the toggles it passes
          (`useToggleDrag`); every other press goes on to the list. Caught
          on the way down, as a row's own press handling stops it rising. */}
      <div onPointerDownCapture={toggles.onPointerDown} role="presentation">
        <ListBox
          aria-label={label}
          autoFocus={focusContents}
          className="w-full overflow-visible"
          ref={listboxRef}
          {...listProps}
        >
          {hasSections
            ? renderGroups(content.items)
            : content.items.map(renderItem)}
        </ListBox>
      </div>
    </ScrollArea>
  );
}
