// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What every editor island story shares: the canvas, and whether the island
// is out, so one control shows any part coming out and tucking away, as the
// other islands' stories do.

export type EditorIslandStoryProps = { isShown?: boolean };

export const editorIslandStoryMeta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  parameters: {
    controls: { include: ["isShown"] },
    layout: "fullscreen",
  },
};
