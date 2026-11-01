// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What a `ScrollArea` changes of its own as it scrolls: the viewport's style,
// whose edge fade is rewritten at every scroll, and the scrollbars, whose
// classes come and go as they show and hide. None of it sizes what holds
// the area, so what watches content for size changes leaves it be: measuring
// at every scroll also puts the scroll back where it was read, and WebKit
// lets that undo the gesture under way, the content jumping as it scrolls.
const CHROME =
  "[data-overlayscrollbars-viewport], .os-scrollbar, .os-scrollbar *";

/** Whether `node` is a scroll area's own chrome rather than its content. */
export const isScrollAreaChrome = (node: Node) =>
  node instanceof Element && node.matches(CHROME);
