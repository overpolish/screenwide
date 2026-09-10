// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import {
  AppWindowMac,
  ArrowBigDownDash,
  ClipboardCopy,
  ImageDown,
  Volume2,
} from "lucide-react";
import { ReactNode, useLayoutEffect, useRef } from "react";
import {
  Header,
  ListBoxSection as AriaListBoxSection,
} from "react-aria-components";

import { Badge } from "../../components/base/badge/badge";
import { ListBox } from "../../components/base/listbox/listbox";
import { ListBoxItem } from "../../components/base/listbox-item/listbox-item";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { Text } from "../../components/base/text/text";

import { hidePopupPanel } from "./api";
import { emptyPopupPanelHeight, popupPanelMaxHeight } from "./layout";
import { PopupPanelIcon, PopupPanelItem, usePopupPanelStore } from "./store";

// Items cross a window boundary as data, so a glyph travels by name.
const itemGlyphs = {
  clipboard: ClipboardCopy,
  image: ImageDown,
  scrolling: ArrowBigDownDash,
};

function ItemGlyph({ icon }: { icon: PopupPanelIcon }) {
  const Glyph = itemGlyphs[icon];
  return <Glyph />;
}

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

export function PopupPanelWindow() {
  const active = usePopupPanelStore((state) => state.active);
  const close = usePopupPanelStore((state) => state.close);
  const select = usePopupPanelStore((state) => state.select);
  const listboxRef = useRef<HTMLDivElement>(null);
  const selectingRef = useRef(false);
  const activeId = active?.id;
  const activeItemCount = active?.items.length ?? 0;

  useLayoutEffect(() => {
    if (activeId == null || !listboxRef.current) return;

    const window = getCurrentWindow();
    const listbox = listboxRef.current;
    const scrollContent = listbox.parentElement;
    let cancelled = false;

    const resize = async () => {
      const scaleFactor = await window.scaleFactor();
      const currentSize = (await window.innerSize()).toLogical(scaleFactor);
      if (cancelled) return;

      const height =
        activeItemCount === 0
          ? emptyPopupPanelHeight
          : Math.min(
              scrollContent?.scrollHeight ?? listbox.scrollHeight,
              popupPanelMaxHeight,
            );
      await window.setSize(new LogicalSize(currentSize.width, height));
    };

    void resize();

    const observer = new ResizeObserver(() => {
      void resize();
    });
    observer.observe(scrollContent ?? listbox);

    return () => {
      cancelled = true;
      observer.disconnect();
    };
  }, [activeId, activeItemCount]);

  if (!active) return null;

  // A menu runs actions rather than holding a pick, so nothing is ever drawn
  // as selected and the gutter that would carry the checkmark is dropped.
  const showsSelection = active.mode === "select";

  if (active.items.length === 0) {
    return (
      <div
        className="window-surface rounded-window px-section flex h-full min-h-16 w-full items-center justify-center overflow-hidden text-center text-xs text-muted"
        ref={listboxRef}
      >
        No options available
      </div>
    );
  }

  const selectItem = (item: PopupPanelItem) => {
    if (selectingRef.current) return;

    // A toggle flips its own tick in the list it belongs to and stays open;
    // the listener reads which item was pressed rather than the tick list.
    if (item.togglesInPlace) {
      const selectedIds = active.selectedIds.includes(item.id)
        ? active.selectedIds.filter((id) => id !== item.id)
        : [...active.selectedIds, item.id];
      select(active.id, selectedIds, item.id);
      return;
    }

    selectingRef.current = true;
    select(active.id, [item.id], item.id);
    close();
    void hidePopupPanel(active.focusContents).finally(() => {
      selectingRef.current = false;
    });
  };

  const onSelectionChange = (selection: "all" | Set<number | string>) => {
    if (selection === "all") return;
    // A single choice is reported by the item's own press, which also fires
    // for the item already chosen; reporting it here as well sent every press
    // twice, which flipped a toggle straight back.
    if (active.selectionMode === "single") return;

    const selectedIds = new Set(
      [...selection].map((selectedId) => selectedId.toString()),
    );
    const exclusiveId = active.exclusiveId;
    const previouslyExclusive = exclusiveId
      ? active.selectedIds.includes(exclusiveId)
      : false;
    if (exclusiveId && selectedIds.has(exclusiveId) && !previouslyExclusive) {
      select(active.id, [exclusiveId]);
      return;
    }
    if (exclusiveId) selectedIds.delete(exclusiveId);
    if (selectedIds.size === 0 && exclusiveId) selectedIds.add(exclusiveId);
    select(
      active.id,
      active.items
        .map((item) => item.id)
        .filter((itemId) => selectedIds.has(itemId)),
    );
  };

  const renderItem = (item: PopupPanelItem): ReactNode => (
    <ListBoxItem
      id={item.id}
      key={item.id}
      onPress={() => {
        if (active.selectionMode === "single") selectItem(item);
      }}
      showsSelection={showsSelection}
      textValue={item.label}
    >
      <span className="flex w-full min-w-0 items-center gap-control-inset [&_svg]:size-icon [&_svg]:shrink-0">
        {item.iconPath ? (
          <img
            alt=""
            className="size-icon shrink-0 object-contain"
            src={convertFileSrc(item.iconPath)}
          />
        ) : item.icon ? (
          <ItemGlyph icon={item.icon} />
        ) : item.id === active.exclusiveId ? (
          <Volume2 />
        ) : active.selectionMode === "multiple" ? (
          <AppWindowMac />
        ) : null}
        <span className="truncate">{item.label}</span>
        {item.detail ? <Badge className="ml-auto">{item.detail}</Badge> : null}
      </span>
    </ListBoxItem>
  );

  const hasSections = active.items.some((item) => item.section !== undefined);

  return (
    <ScrollArea
      key={active.id}
      // The scrollbar keeps out of the rounded corners.
      rootClassName="window-surface rounded-window [--scrollbar-inset:var(--radius-window)]"
    >
      <ListBox
        aria-label={active.label}
        autoFocus={active.focusContents}
        className="w-full overflow-visible"
        onSelectionChange={onSelectionChange}
        ref={listboxRef}
        selectedKeys={showsSelection ? active.selectedIds : []}
        selectionBehavior="toggle"
        selectionMode={active.selectionMode}
      >
        {hasSections
          ? groupItemsBySection(active.items).map((group) => (
              <AriaListBoxSection
                className="flex flex-col"
                id={group.id}
                key={group.id}
              >
                {group.section ? (
                  <Header>
                    <Text
                      as="span"
                      className="px-control-inset py-control block"
                      variant="section"
                    >
                      {group.section}
                    </Text>
                  </Header>
                ) : null}
                {group.items.map(renderItem)}
              </AriaListBoxSection>
            ))
          : active.items.map(renderItem)}
      </ListBox>
    </ScrollArea>
  );
}
