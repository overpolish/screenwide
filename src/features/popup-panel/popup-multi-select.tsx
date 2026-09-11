// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import { ReactNode, useEffect, useMemo, useRef } from "react";

import { ListBoxItem } from "../../components/base/listbox-item/listbox-item";
import { Select } from "../../components/base/select/select";

import { hidePopupPanel, showPopupPanel } from "./api";
import { initialPopupPanelHeight } from "./layout";
import { PopupPanelItem, usePopupPanelStore } from "./store";

type PopupMultiSelectProps = {
  exclusiveId: string;
  id: string;
  items: PopupPanelItem[];
  label: string;
  onSelectionChange: (items: PopupPanelItem[]) => void;
  placeholder: string;
  selectedIds: string[];
  leftSection?: ReactNode;
  onOpen?: () => Promise<PopupPanelItem[]>;
};

export function PopupMultiSelect({
  exclusiveId,
  id,
  items,
  label,
  leftSection,
  onOpen,
  onSelectionChange,
  placeholder,
  selectedIds,
}: PopupMultiSelectProps) {
  const active = usePopupPanelStore((state) => state.active);
  const close = usePopupPanelStore((state) => state.close);
  const lastSelection = usePopupPanelStore((state) => state.lastSelection);
  const open = usePopupPanelStore((state) => state.open);
  const handledEventRef = useRef(lastSelection?.eventId ?? null);
  const triggerRef = useRef<HTMLButtonElement>(null);

  const selectedItems = useMemo(
    () => items.filter((item) => selectedIds.includes(item.id)),
    [items, selectedIds],
  );
  const triggerItem = useMemo<PopupPanelItem | null>(() => {
    if (selectedItems.length === 0) return null;
    if (selectedItems.length === 1) return selectedItems[0];
    return {
      id: selectedItems.map((item) => item.id).join(","),
      label: `${selectedItems.length.toString()} applications`,
    };
  }, [selectedItems]);

  useEffect(() => {
    if (
      !lastSelection ||
      lastSelection.id !== id ||
      lastSelection.eventId === handledEventRef.current
    ) {
      return;
    }

    handledEventRef.current = lastSelection.eventId;
    onSelectionChange(
      items.filter((item) => lastSelection.selectedIds.includes(item.id)),
    );
  }, [id, items, lastSelection, onSelectionChange]);

  const showListbox = async (focusContents: boolean) => {
    const trigger = triggerRef.current;
    if (!trigger) return;

    const bounds = trigger.getBoundingClientRect();
    const currentItems = onOpen ? await onOpen() : items;
    const height = initialPopupPanelHeight(currentItems.length);

    open({
      content: {
        exclusiveId,
        items: currentItems,
        kind: "list",
        mode: "select",
        selectedIds,
        selectionMode: "multiple",
      },
      focusContents,
      id,
      label,
    });
    await showPopupPanel({
      anchor: {
        height: bounds.height,
        width: bounds.width,
        x: bounds.left,
        y: bounds.top,
      },
      focusContents,
      offset: new LogicalPosition(bounds.left, bounds.bottom + 4),
      parentWindowLabel: getCurrentWindow().label,
      size: new LogicalSize(bounds.width, height),
      triggerId: id,
    });
  };

  const toggleListbox = async (focusContents: boolean) => {
    const current = usePopupPanelStore.getState().active;
    const isOpen = current?.id === id;
    if (isOpen) {
      close();
      await hidePopupPanel(current.focusContents);
    } else {
      await showListbox(focusContents);
    }
  };

  return (
    <div
      className="w-full"
      data-popup-panel-trigger={id}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
    >
      <Select
        aria-label={label}
        className="w-full"
        clearable={false}
        isOpen={active?.id === id}
        items={triggerItem ? [triggerItem] : []}
        leftSection={leftSection}
        onPress={(event) => {
          void toggleListbox(
            ["keyboard", "virtual"].includes(event.pointerType),
          );
        }}
        placeholder={placeholder}
        standalone
        triggerRef={triggerRef}
        value={triggerItem?.id ?? null}
      >
        {(item: PopupPanelItem) => (
          <ListBoxItem id={item.id} textValue={item.label}>
            {item.label}
          </ListBoxItem>
        )}
      </Select>
    </div>
  );
}
