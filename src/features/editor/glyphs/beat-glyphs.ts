// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AppWindow,
  Flag,
  Keyboard,
  LayoutPanelTop,
  type LucideIcon,
  MessageSquareQuote,
  MousePointerClick,
  TextCursorInput,
  Volume2,
} from "lucide-react";

// What happens at a beat, drawn the same on the strip and on its island.

export const BEAT_GLYPHS = {
  click: MousePointerClick,
  keys: Keyboard,
  moment: Flag,
  screen: LayoutPanelTop,
  sound: Volume2,
  speech: MessageSquareQuote,
  switch: AppWindow,
  type: TextCursorInput,
} satisfies Record<string, LucideIcon>;
