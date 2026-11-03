// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn, userEvent, within } from "storybook/test";

import { t } from "../../../i18n/i18n";
import { ISLAND } from "../../../primitives/island/island-geometry";
import { featureStage } from "../../../storybook/feature-story-stage";
import { PREVIEW_NOTCH } from "../../../storybook/preview-notch";
import { StoryNotch } from "../../../storybook/story-notch";

import { TextRecognitionIsland } from "./text-recognition-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

// While an area is drawn the island is at the top of the screen, under the
// notch from the toolbar when it is on; from then on it hangs under the area
// it reads, drawn here as the dashed box above it.

const meta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: {
    isShown: true,
    onAction: fn(),
    onCopyQrCode: fn(() => Promise.resolve()),
    onOpenQrCode: fn(() => Promise.resolve()),
    state: { failed: false, phase: "choosing" },
  },
  component: TextRecognitionIsland,
  decorators: [
    (Story, { args, globals }) =>
      Story({
        args: {
          ...args,
          notch:
            args.state.phase === "choosing" && globals.notch !== "off"
              ? PREVIEW_NOTCH
              : null,
        },
      }),
    featureStage({ height: 320, width: 720 }),
  ],
  parameters: {
    controls: { include: ["isShown"] },
    layout: "fullscreen",
    notch: true,
  },
  render: (args) =>
    args.state.phase === "choosing" ? (
      <div className="relative size-full" dir="ltr">
        <div
          className="absolute inset-x-0"
          style={{ top: args.notch ? 0 : ISLAND.screenGap }}
        >
          <TextRecognitionIsland {...args} />
        </div>
        {args.notch ? <StoryNotch notch={args.notch} /> : null}
      </div>
    ) : (
      <div className="flex size-full flex-col items-center gap-separate pt-12">
        <div className="h-36 w-96 rounded-sm border border-dashed border-info" />
        <div className="w-full">
          <TextRecognitionIsland {...args} />
        </div>
      </div>
    ),
  title: "Features/Text Recognition/Island/Phases",
} satisfies Meta<typeof TextRecognitionIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Choosing: Story = {};

export const Failed: Story = {
  args: { state: { failed: true, phase: "choosing" } },
};

export const Reading: Story = {
  args: { state: { phase: "reading" } },
};

export const Found: Story = {
  args: { state: { lines: 3, phase: "ready", qrCodes: 1, selected: false } },
};

export const TextSelected: Story = {
  args: { state: { lines: 3, phase: "ready", qrCodes: 0, selected: true } },
};

export const OnlyQrCodes: Story = {
  args: { state: { lines: 0, phase: "ready", qrCodes: 2, selected: false } },
};

export const NothingFound: Story = {
  args: { state: { lines: 0, phase: "ready", qrCodes: 0, selected: false } },
};

export const QrLink: Story = {
  args: {
    state: {
      content: "https://screenwide.app/download",
      decodeError: null,
      phase: "qr",
    },
  },
};

export const QrInformation: Story = {
  args: {
    state: {
      content:
        "BEGIN:VCARD\nVERSION:3.0\nFN:Ada Lovelace\nORG:Analytical Engines Ltd\nTEL:+44 20 7946 0958\nEMAIL:ada@example.com\nADR:;;12 St James's Square;London;;SW1Y 4JH;United Kingdom\nNOTE:Met at the engine demonstration. Ask about the notes on Bernoulli numbers.\nEND:VCARD",
      decodeError: null,
      phase: "qr",
    },
  },
};

export const QrUnsupported: Story = {
  args: {
    state: { content: "javascript:alert(1)", decodeError: null, phase: "qr" },
  },
};

export const QrDecodeFailure: Story = {
  args: {
    state: {
      content: "",
      decodeError: "QR-like code could not be decoded.",
      phase: "qr",
    },
  },
};

export const QrOpenFailure: Story = {
  args: {
    ...QrLink.args,
    onOpenQrCode: fn(() => Promise.reject(new Error("No browser"))),
  },
  play: async ({ canvasElement }) => {
    await userEvent.click(
      await within(canvasElement).findByRole("button", {
        name: t("text-recognition-qr-open-link"),
      }),
    );
  },
};

export const QrCopyFailure: Story = {
  args: {
    ...QrLink.args,
    onCopyQrCode: fn(() => Promise.reject(new Error("Clipboard busy"))),
  },
  play: async ({ canvasElement }) => {
    await userEvent.click(
      await within(canvasElement).findByRole("button", {
        name: t("text-recognition-island-qr-copy"),
      }),
    );
  },
};
