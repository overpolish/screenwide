// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { SkinContext } from "../../../../primitives/skin/skin";

import type { Decorator } from "@storybook/react-vite";

/** Draws a story as the cue picker's panel window holds it: on the island's
 * black and its faint ring, as `popup-panel-window.tsx` paints it, in the
 * island's skin the picker takes. */
export const panelStory: Decorator = (Story) => (
  <div className="dark size-full overflow-hidden rounded-island bg-island text-fg inset-ring inset-ring-fill-strong">
    <SkinContext value="island">
      <Story />
    </SkinContext>
  </div>
);
