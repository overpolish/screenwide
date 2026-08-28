// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { AppWindowMac, Volume2 } from "lucide-react";
import { useLayoutEffect, useRef } from "react";

import { ListBox } from "../../components/base/listbox/listbox";
import { ListBoxItem } from "../../components/base/listbox-item/listbox-item";
import { OverflowShadow } from "../../components/base/overflow-shadow/overflow-shadow";

import { hideStandaloneListbox } from "./api";
import {
  emptyStandaloneListboxHeight,
  standaloneListboxMaxHeight,
} from "./layout";
import { useStandaloneListboxStore } from "./store";

export function StandaloneListboxWindow() {
  const active = useStandaloneListboxStore((state) => state.active);
  const close = useStandaloneListboxStore((state) => state.close);
  const select = useStandaloneListboxStore((state) => state.select);
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
          ? emptyStandaloneListboxHeight
          : Math.min(
              scrollContent?.scrollHeight ?? listbox.scrollHeight,
              standaloneListboxMaxHeight,
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

  const selectItem = (selectedId: number | string) => {
    if (selectingRef.current) return;

    selectingRef.current = true;
    select(active.id, [selectedId.toString()]);
    close();
    void hideStandaloneListbox(active.focusContents).finally(() => {
      selectingRef.current = false;
    });
  };

  const onSelectionChange = (selection: "all" | Set<number | string>) => {
    if (selection === "all") return;
    if (active.selectionMode === "single") {
      const selected = selection.values().next();
      if (!selected.done) {
        select(active.id, [selected.value.toString()]);
      }
      return;
    }

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

  return (
    <OverflowShadow
      key={active.id}
      rootClassName="window-surface rounded-window"
    >
      <ListBox
        aria-label={active.label}
        autoFocus={active.focusContents}
        className="w-full overflow-visible"
        onSelectionChange={onSelectionChange}
        ref={listboxRef}
        selectedKeys={active.selectedIds}
        selectionBehavior="toggle"
        selectionMode={active.selectionMode}
        size="compact"
        variant="transparent"
      >
        {active.items.map((item) => (
          <ListBoxItem
            id={item.id}
            key={item.id}
            onPress={() => {
              if (active.selectionMode === "single") selectItem(item.id);
            }}
            textValue={item.label}
          >
            <span className="gap-control flex min-w-0 items-center">
              {item.iconPath ? (
                <img
                  alt=""
                  className="size-icon-compact shrink-0 object-contain"
                  src={convertFileSrc(item.iconPath)}
                />
              ) : item.id === active.exclusiveId ? (
                <Volume2 className="size-icon-compact shrink-0 text-muted" />
              ) : active.selectionMode === "multiple" ? (
                <AppWindowMac className="size-icon-compact shrink-0 text-muted" />
              ) : null}
              <span className="truncate">{item.label}</span>
            </span>
          </ListBoxItem>
        ))}
      </ListBox>
    </OverflowShadow>
  );
}
