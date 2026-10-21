// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ListBox } from "../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../primitives/listbox-item/listbox-item";
import { MenuSection, PanelWindow } from "../shared/panel-window";

// Everything set once and rarely touched, in one native menu beside Record:
// the countdown, what the recording shows of the pointer and keyboard, how
// screenshots are kept, and the camera's own settings. A menu of checked
// items, as a native one is.

const SECTIONS = [
  {
    items: ["No countdown", "3 seconds", "10 seconds"],
    title: "Countdown",
  },
  { items: ["Show clicks", "Show keys pressed"], title: "Pointer and keys" },
  {
    items: ["Copy instead of saving", "Capture scrolling region"],
    title: "Screenshots",
  },
  { items: ["Flip horizontally", "Anti-flicker 50 Hz"], title: "Camera" },
];

export function RecorderOptions() {
  return (
    <PanelWindow className="w-60">
      <ListBox
        aria-label="Options"
        className="w-full"
        defaultSelectedKeys={["No countdown", "Show clicks"]}
        selectionMode="multiple"
      >
        {SECTIONS.map(({ items, title }) => (
          <MenuSection key={title} title={title}>
            {items.map((item) => (
              <ListBoxItem id={item} key={item}>
                {item}
              </ListBoxItem>
            ))}
          </MenuSection>
        ))}
      </ListBox>
    </PanelWindow>
  );
}
