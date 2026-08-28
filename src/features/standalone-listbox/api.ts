// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";

type ShowStandaloneListboxOptions = {
  focusContents: boolean;
  offset: LogicalPosition;
  parentWindowLabel: string;
  size: LogicalSize;
  triggerId: string;
};

export const showStandaloneListbox = ({
  focusContents,
  offset,
  parentWindowLabel,
  size,
  triggerId,
}: ShowStandaloneListboxOptions) =>
  invoke<null>("show_standalone_listbox", {
    focusContents,
    offset,
    parentWindowLabel,
    size,
    triggerId,
  });

export const hideStandaloneListbox = (returnFocus = false) =>
  invoke<null>("hide_standalone_listbox", { returnFocus });
