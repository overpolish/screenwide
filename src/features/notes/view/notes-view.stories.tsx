// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";
import { fn, userEvent, within } from "storybook/test";

import { t } from "../../../i18n/i18n";
import { featureStage } from "../../../storybook/feature-story-stage";
import { parseNotes, renameSpeaker } from "../document/notes-document";
import {
  type SensitiveLookup,
  sensitiveByText,
} from "../sensitive/use-sensitive";

import { NotesView } from "./notes-view";
import { useNotesHistory } from "./use-notes-history";

import type { SensitiveFindKind } from "../../../bindings/SensitiveFindKind";
import type { Meta, StoryObj } from "@storybook/react-vite";

const NOTES = parseNotes(`# Notes 2026-05-04 at 10.30.00

**Me:** Thanks for joining. Let's go through the launch plan for the beta.

**Speaker 1:** The build is ready. We are waiting on the store review, which usually takes two days.

**Speaker 1:** If it comes back clean we can ship on Monday.

**Speaker 2:** Marketing needs the final screenshots by Thursday, and the release notes a day before.

**Me:** I can send those tomorrow morning.

**Speaker 1:** Great. Then we ship on Monday.
`);

const PRIVATE_NOTES = parseNotes(`# Notes 2026-05-04 at 14.00.00

**Me:** Thanks for joining. Let's go through the renewal.

**Speaker 1:** I spoke to Priya Shah at Northwind Traders yesterday and she's happy with the quote.

**Speaker 1:** Send the contract to dana.scully@fbi-records.com, or call her on 07700 900123 if anything comes up.

**Speaker 2:** The invoice goes to 221B Baker Street, London NW1 6XE, and they pay into sort code 20-00-00.

**Me:** Priya Shah signs off on Friday, then. She's pregnant, so we'll need cover from May.
`);

/** What the app would find in the notes above, found wherever the words
 * still stand so edits and redactions update the marks as they would. */
const PRIVATE: [string, SensitiveFindKind][] = [
  ["Priya Shah", "name"],
  ["Northwind Traders", "company"],
  ["dana.scully@fbi-records.com", "email"],
  ["07700 900123", "phone"],
  ["221B Baker Street, London NW1 6XE", "address"],
  ["20-00-00", "account"],
  ["pregnant", "health"],
];
const privateIn: SensitiveLookup = (_, text) =>
  PRIVATE.flatMap(([words, kind]) => {
    const start = text.indexOf(words);
    return start === -1 ? [] : [{ end: start + words.length, kind, start }];
  }).sort((a, b) => a.start - b.start);

const meta = {
  args: {
    canRedo: false,
    canUndo: false,
    document: NOTES,
    isTaking: false,
    onBack: fn(),
    onChange: fn(),
    onClose: fn(),
    onCopyPath: fn(),
    onHearVoice: fn(),
    onMinimize: fn(),
    onRedo: fn(),
    onRename: fn(),
    onUndo: fn(),
    playingVoice: null,
    sensitive: sensitiveByText(new Map()),
    title: "Notes 2026-05-04 at 10.30.00",
    voices: [1, 2],
  },
  component: NotesView,
  decorators: [featureStage({ window: "projects" })],
  parameters: {
    controls: {
      exclude: [
        "canRedo",
        "canUndo",
        "document",
        "onBack",
        "onChange",
        "onClose",
        "onCopyPath",
        "onHearVoice",
        "onMinimize",
        "onRedo",
        "onRename",
        "onUndo",
        "sensitive",
      ],
    },
  },
  // Edits, renames, Undo and the voice playing stay in the story, as they
  // would in the app.
  render: function Render(args) {
    const history = useNotesHistory();
    const { reset } = history;
    const [playing, setPlaying] = useState(args.playingVoice);
    useEffect(() => {
      reset(args.document);
    }, [args.document, reset]);
    return (
      <NotesView
        {...args}
        canRedo={history.canRedo}
        canUndo={history.canUndo}
        document={history.document}
        onChange={(document, typing) => {
          history.change(document, typing);
          args.onChange(document, typing);
        }}
        onHearVoice={(number) => {
          setPlaying((current) => (current === number ? null : number));
          args.onHearVoice(number);
        }}
        onRedo={() => {
          history.redo();
          args.onRedo();
        }}
        onUndo={() => {
          history.undo();
          args.onUndo();
        }}
        playingVoice={playing}
      />
    );
  },
  title: "Features/Notes/View/States",
} satisfies Meta<typeof NotesView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const StillTaking: Story = { args: { isTaking: true } };

export const SaveFailed: Story = {
  args: { saveError: "The disk is full" },
};

export const Replacing: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.type(
      await canvas.findByRole("searchbox", { name: t("notes-view-search") }),
      "monday{Enter}",
    );
    await userEvent.click(
      await canvas.findByRole("button", { name: t("notes-view-find") }),
    );
  },
};

export const Searching: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.type(
      await canvas.findByRole("searchbox", { name: t("notes-view-search") }),
      "monday{Enter}",
    );
  },
};

export const Filtered: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(
      await canvas.findByRole("button", {
        name: t("notes-view-speaker-filter", {
          share: "50%",
          speaker: "Speaker 1",
        }),
      }),
    );
  },
};

export const NamedSpeakers: Story = {
  args: {
    document: renameSpeaker(
      renameSpeaker(NOTES, "Speaker 1", "Priya Shah"),
      "Speaker 2",
      "Tom",
    ),
    voices: [],
  },
};

export const Sensitive: Story = {
  args: {
    document: PRIVATE_NOTES,
    sensitive: privateIn,
    title: "Notes 2026-05-04 at 14.00.00",
    voices: [],
  },
};
