// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { libraryIsland, type LibraryIslandProps } from "./library-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function LibraryIsland({
  isShown,
  ...props
}: LibraryIslandProps & EditorIslandStoryProps) {
  return <EditorIslandStage content={libraryIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canDrag: true,
    hasSongRoom: true,
    isShown: true,
    kinds: ["music", "effect", "image"],
    onAddPicture: fn(),
    onAddSound: fn(),
  },
  component: LibraryIsland,
  decorators: [featureStage({ height: 520, width: 560 })],
  parameters: { controls: { exclude: ["kinds"] } },
  title: "Features/Editor/Island/Library",
} satisfies Meta<typeof LibraryIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// The music track is full: pressing a song adds nothing.
export const NoSongRoom: Story = {
  args: { hasSongRoom: false },
};

// A screenshot's library, which holds only pictures.
export const Screenshot: Story = {
  args: { kinds: ["image"] },
};
