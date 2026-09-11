// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import { ReactNode, useEffect, useRef } from "react";

import { Badge } from "../../components/base/badge/badge";
import { ListBoxItem } from "../../components/base/listbox-item/listbox-item";
import { Select } from "../../components/base/select/select";

import { hidePopupPanel, showPopupPanel } from "./api";
import { initialPopupPanelHeight } from "./layout";
import { PopupPanelItem, usePopupPanelStore } from "./store";

type PopupSelectProps = {
  id: string;
  items: PopupPanelItem[];
  label: string;
  onSelectionChange: (item: PopupPanelItem) => void;
  placeholder: string;
  selectedId: string | null;
  /** A form row's title and note, when the pop-up sits in one. */
  "aria-describedby"?: string;
  "aria-labelledby"?: string;
  isDisabled?: boolean;
  leftSection?: ReactNode;
  onOpen?: () => Promise<PopupPanelItem[]>;
};

export function PopupSelect({
  "aria-describedby": ariaDescribedBy,
  "aria-labelledby": ariaLabelledBy,
  id,
  isDisabled = false,
  items,
  label,
  leftSection,
  onOpen,
  onSelectionChange,
  placeholder,
  selectedId,
}: PopupSelectProps) {
  const active = usePopupPanelStore((state) => state.active);
  const lastSelection = usePopupPanelStore((state) => state.lastSelection);
  const close = usePopupPanelStore((state) => state.close);
  const open = usePopupPanelStore((state) => state.open);
  const handledEventRef = useRef(lastSelection?.eventId ?? null);
  const triggerRef = useRef<HTMLButtonElement>(null);

  const selectedItem = items.find((item) => item.id === selectedId) ?? null;

  useEffect(() => {
    if (
      !lastSelection ||
      lastSelection.id !== id ||
      lastSelection.eventId === handledEventRef.current
    ) {
      return;
    }

    handledEventRef.current = lastSelection.eventId;
    const item = items.find(
      (candidate) => candidate.id === lastSelection.selectedIds[0],
    );
    if (item) onSelectionChange(item);
  }, [id, items, lastSelection, onSelectionChange]);

  const showListbox = async (focusContents: boolean) => {
    const trigger = triggerRef.current;
    if (!trigger) return;

    const bounds = trigger.getBoundingClientRect();
    const currentItems = onOpen ? await onOpen() : items;
    const height = initialPopupPanelHeight(currentItems.length);

    open({
      content: {
        items: currentItems,
        kind: "list",
        mode: "select",
        selectedIds: selectedId ? [selectedId] : [],
        selectionMode: "single",
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
        aria-describedby={ariaDescribedBy}
        aria-label={ariaLabelledBy ? undefined : label}
        aria-labelledby={ariaLabelledBy}
        className="w-full"
        clearable={false}
        isDisabled={isDisabled}
        isOpen={active?.id === id}
        items={selectedItem ? [selectedItem] : []}
        leftSection={leftSection}
        onPress={(event) => {
          void toggleListbox(
            ["keyboard", "virtual"].includes(event.pointerType),
          );
        }}
        placeholder={placeholder}
        renderValue={(item) =>
          item ? (
            <span className="flex w-full items-center justify-between gap-control">
              <span className="truncate">{item.label}</span>
              {item.detail ? <Badge>{item.detail}</Badge> : null}
            </span>
          ) : null
        }
        standalone
        triggerRef={triggerRef}
        value={selectedId}
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
