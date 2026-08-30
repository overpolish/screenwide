// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Meta, type StoryObj } from "@storybook/react-vite";

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { QrDetails } from "./qr-details";

const meta = {
  args: {
    content: "https://screenwide.app",
    onAction: () => undefined,
    onClose: () => undefined,
    onCopy: () => undefined,
    payload: {
      action: "link",
      kind: "action",
      label: "Open link",
      url: "https://screenwide.app/",
    },
  },
  component: QrDetails,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage height={360} viewMode={context.viewMode} width={480}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/QR Details",
} satisfies Meta<typeof QrDetails>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ActionableLink: Story = {};

export const Information: Story = {
  args: {
    content: "WIFI:T:WPA;S:Studio;P:correct-horse-battery-staple;;",
    onAction: undefined,
    payload: { kind: "information", label: "Wi-Fi network" },
  },
};

export const Unsupported: Story = {
  args: {
    content: "javascript:alert(1)",
    onAction: undefined,
    payload: {
      kind: "unsupported",
      label: "Unsupported QR",
      reason: "The javascript action is not supported.",
    },
  },
};

export const DecodeFailure: Story = {
  args: {
    content: "",
    onAction: undefined,
    payload: {
      kind: "unsupported",
      label: "Unsupported QR",
      reason: "QR-like code could not be decoded.",
    },
  },
};

export const OpenFailure: Story = {
  args: { error: "Could not open link." },
};

export const CopyFailure: Story = {
  args: { error: "Could not copy QR content." },
};
