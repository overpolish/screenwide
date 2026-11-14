// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { expect, fn, userEvent, waitFor, within } from "storybook/test";

import { t } from "../../../../i18n/i18n";
import { featureStage } from "../../../../storybook/feature-story-stage";
import {
  answerOnlineSearch,
  type StoryOnlineSearch,
} from "../../../../storybook/story-online-search";
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

const QUERY = "piano";

/** A story that searches online for `QUERY`, answered as `answer`, then
 * waits in the canvas `element` until `shown` finds what it shows. */
const searchedOnline = (
  answer: StoryOnlineSearch,
  shown: (element: HTMLElement) => Promise<unknown>,
): Story => ({
  beforeEach: () => answerOnlineSearch(answer),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.type(
      await canvas.findByRole("searchbox", {
        name: t("editor-island-sound-library-search"),
      }),
      QUERY,
    );
    await userEvent.click(
      await canvas.findByRole("button", {
        name: t("editor-island-openverse-search", { query: QUERY }),
      }),
    );
    await shown(canvasElement);
  },
});

export const OnlineResults: Story = searchedOnline("found", (element) =>
  within(element).findByText("Soft Piano Theme"),
);

export const SearchingOnline: Story = searchedOnline("searching", (element) =>
  within(element).findByRole("progressbar", {
    name: t("editor-island-openverse-searching"),
  }),
);

export const NothingFoundOnline: Story = searchedOnline("nothing", (element) =>
  within(element).findByText(t("editor-island-openverse-none")),
);

// Asked too often this minute: it can be tried again at once.
export const TooManySearches: Story = searchedOnline("busy", (element) =>
  within(element).findByText(t("editor-island-openverse-busy")),
);

// Today's searches used up: they come back when the day ends, shown in the
// viewer's own time, and nothing is offered to press.
export const SearchesUsedUp: Story = searchedOnline(
  "usedUp",
  async (element) => {
    const canvas = within(element);
    await waitFor(() =>
      expect(
        canvas.getByText((text) =>
          text.startsWith("You've reached today's search limit"),
        ),
      ).toBeVisible(),
    );
    await expect(
      canvas.queryByRole("button", {
        name: t("editor-island-openverse-retry"),
      }),
    ).toBeNull();
  },
);

export const SearchNotAnswering: Story = searchedOnline(
  "unavailable",
  (element) =>
    within(element).findByText(t("editor-island-openverse-unavailable")),
);

// Looked up again as it downloads, the song is no longer offered under a
// licence Screenwide may use, so it is not added.
export const DownloadWithdrawn: Story = searchedOnline(
  "withdrawn",
  async (element) => {
    const canvas = within(element);
    const [download] = await canvas.findAllByRole("button", {
      name: (name) =>
        name.startsWith("Download") && name.includes("Morning Walk"),
    });
    await userEvent.click(download);
    await canvas.findByText(
      t("editor-island-openverse-withdrawn", { title: "Morning Walk" }),
      undefined,
      { timeout: 5_000 },
    );
  },
);
