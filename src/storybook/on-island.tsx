// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SkinContext } from "../primitives/skin/skin";

import type { Decorator } from "@storybook/react-vite";

/** Draws a story as the island holds it: on the island's black, which is
 * always dark, in the island's skin. `width` is the width of a section, for
 * parts that sit in one. */
export const onIsland =
  (width?: string): Decorator =>
  (Story) => (
    <div
      className="dark rounded-panel bg-island p-inset text-fg"
      style={{ width }}
    >
      <SkinContext value="island">
        <Story />
      </SkinContext>
    </div>
  );
