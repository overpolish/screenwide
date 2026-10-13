// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { AppWindowMac, Volume2 } from "lucide-react";
import { CSSProperties } from "react";

import { Badge } from "../../components/base/badge/badge";
import { Keyboard, Shortcut } from "../../components/base/keyboard/keyboard";
import { ListBoxItem } from "../../components/base/listbox-item/listbox-item";
import { hotkeyKeys } from "../../components/shared/hotkey-field/hotkey";

import { PopupPanelItemGlyph } from "./popup-panel-item-glyph";
import { PopupPanelItem, PopupPanelListContent } from "./store";
import { POPUP_LABEL_ATTRIBUTE } from "./use-fitted-list-window";

/** The typeface an item's label is set in, where it shows one. */
export const itemFontStyle = (
  font: PopupPanelItem["font"],
): CSSProperties | undefined =>
  font
    ? { fontFamily: `"${font.family}"`, fontWeight: font.weight }
    : undefined;

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
    id={item.id}
    key={item.id}
    onPress={() => {
      if (content.selectionMode === "single") onPress(item);
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
        <PopupPanelItemGlyph icon={item.icon} />
      ) : item.id === content.exclusiveId ? (
        <Volume2 />
      ) : content.selectionMode === "multiple" ? (
        <AppWindowMac />
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
        <Shortcut className="ms-auto">
          {hotkeyKeys(item.shortcut).map((key, index) => (
            <Keyboard key={`${key}-${index.toString()}`}>{key}</Keyboard>
          ))}
        </Shortcut>
      ) : null}
    </span>
  </ListBoxItem>
);
