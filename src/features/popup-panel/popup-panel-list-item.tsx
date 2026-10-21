// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { AppWindowMac, Volume2 } from "lucide-react";
import { CSSProperties } from "react";

import { hotkeyKeys } from "../../lib/hotkey";
import { isWindowsHost } from "../../lib/platform";
import { Badge } from "../../primitives/badge/badge";
import { Keyboard, Shortcut } from "../../primitives/keyboard/keyboard";
import { ListBoxItem } from "../../primitives/listbox-item/listbox-item";

import { PopupPanelItemGlyph } from "./popup-panel-item-glyph";
import { PopupPanelItem, PopupPanelListContent } from "./store";
import { POPUP_LABEL_ATTRIBUTE } from "./use-fitted-list-window";

/** The typeface an item's label is set in, where it shows one. */
export const itemFontStyle = (
  font: PopupPanelItem["font"],
): CSSProperties | undefined =>
  font
    ? {
        fontFamily: `"${font.family}"`,
        fontStyle: font.italic ? "italic" : undefined,
        fontWeight: font.weight,
      }
    : undefined;

// A menu's glyph at the platform's size. macOS 26 draws a menu item's SF
// Symbol at 13pt in the small scale (trash in a 13 by 14 box), which Lucide
// matches at 14px; WinUI flyouts and Explorer menus use 16px icons.
const glyphClassName = "size-icon-small shrink-0 windows:size-icon";

// Keys as the platform's menus set them: bare, in the secondary label colour.
// A macOS menu turns them white with the label under the highlight and dims
// them with a disabled item. A Fluent flyout keeps them secondary through
// hover and press, a step smaller than the label, joined by plus signs.
const shortcutClassName = [
  "ms-auto gap-tight text-content-fg-secondary",
  "group-data-[hovered]:text-primary-fg group-data-[focus-visible]:text-primary-fg group-data-[pressed]:text-primary-fg",
  "group-data-[disabled]:text-control-fg-disabled",
  "windows:gap-0 windows:text-subheadline",
  "windows:group-data-[hovered]:text-content-fg-secondary windows:group-data-[focus-visible]:text-content-fg-secondary windows:group-data-[pressed]:text-content-fg-secondary",
].join(" ");

const shortcutKeys = (shortcut: string) =>
  hotkeyKeys(shortcut).flatMap((key, index) => {
    const keycap = (
      <Keyboard key={`${key}-${index.toString()}`} variant="plain">
        {key}
      </Keyboard>
    );
    return index > 0 && isWindowsHost ? ["+" as const, keycap] : [keycap];
  });

/** One row of a pop-up list: its artwork or glyph, its label, and the badge
 * or keys after it. A press on a single choice reaches `onPress`. */
export const renderPopupPanelItem = (
  item: PopupPanelItem,
  {
    content,
    onPress,
    showsSelection,
  }: {
    content: PopupPanelListContent;
    onPress: (item: PopupPanelItem) => void;
    showsSelection: boolean;
  },
) => (
  <ListBoxItem
    className="group"
    id={item.id}
    key={item.id}
    onPress={() => {
      if (content.selectionMode === "single") onPress(item);
    }}
    showsSelection={showsSelection}
    textValue={item.label}
  >
    <span className="flex w-full min-w-0 items-center gap-control-inset">
      {item.iconPath ? (
        <img
          alt=""
          className="size-icon shrink-0 object-contain"
          src={convertFileSrc(item.iconPath)}
        />
      ) : item.icon ? (
        <PopupPanelItemGlyph className={glyphClassName} icon={item.icon} />
      ) : item.id === content.exclusiveId ? (
        <Volume2 className={glyphClassName} />
      ) : content.selectionMode === "multiple" ? (
        <AppWindowMac className={glyphClassName} />
      ) : null}
      {/* Often a device's, a folder's or a font's own name, in whichever
          direction it is written. */}
      <span
        className="truncate"
        dir="auto"
        style={itemFontStyle(item.font)}
        {...{ [POPUP_LABEL_ATTRIBUTE]: "" }}
      >
        {item.label}
      </span>
      {item.detail ? <Badge className="ms-auto">{item.detail}</Badge> : null}
      {item.shortcut ? (
        <Shortcut className={shortcutClassName}>
          {shortcutKeys(item.shortcut)}
        </Shortcut>
      ) : null}
    </span>
  </ListBoxItem>
);
