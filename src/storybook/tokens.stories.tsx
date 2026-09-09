// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ReactNode } from "react";

import { Settings } from "lucide-react";

import { Text, TextProps } from "../components/base/text/text";
import { focusStyles } from "../lib/styling";

import type { Meta, StoryObj } from "@storybook/react-vite";

/**
 * Every design token from `src/index.css` rendered through the Tailwind
 * utilities generated from it, so the page fails visibly if a token is renamed
 * or dropped. Nothing here hardcodes a value: the utility class is both the
 * sample and the label.
 */

const typographyTokens: {
  token: string;
  className?: string;
  variant?: TextProps["variant"];
}[] = [
  { token: "text-title", variant: "title" },
  { token: "text-headline", variant: "headline" },
  { token: "text-body", variant: "body" },
  { token: "text-subheadline", variant: "subheadline" },
  { token: "text-section", variant: "section" },
  { token: "text-footnote", variant: "footnote" },
];

const colorTokens = [
  "bg-content",
  "bg-content-fg",
  "bg-content-fg-secondary",
  "bg-content-fg-tertiary",
  "bg-content-fg-quaternary",
  "bg-primary",
  "bg-primary-surface-hover",
  "bg-primary-surface-pressed",
  "bg-primary-tint",
  "bg-focus-ring",
  "bg-success",
  "bg-info",
  "bg-warning",
  "bg-error",
  "bg-accent-heading-warm",
  "bg-accent-heading",
  "bg-accent-heading-vivid",
];

const contentForegroundTokens = [
  "text-content-fg",
  "text-content-fg-secondary",
  "text-content-fg-tertiary",
  "text-content-fg-quaternary",
];

const fillTokens = [
  "bg-fill",
  "bg-fill-secondary",
  "bg-fill-tertiary",
  "bg-fill-quaternary",
];

const spacingTokens = [
  "w-tight",
  "w-control",
  "w-control-inset",
  "w-section",
  "w-layout",
  "w-window-inset",
  "w-control-height",
  "w-traffic-lights",
];

const radiusTokens = ["rounded-window", "rounded-panel", "rounded-control"];

const iconTokens = [
  "size-icon-mini",
  "size-icon-small",
  "size-icon",
  "size-icon-large",
];

function Group({ children, title }: { children: ReactNode; title: string }) {
  return (
    <section className="flex flex-col gap-section">
      <Text as="h2" variant="headline">
        {title}
      </Text>
      {children}
    </section>
  );
}

function Page({ children }: { children: ReactNode }) {
  return (
    <div className="flex flex-col gap-layout p-window-inset">{children}</div>
  );
}

function Typography() {
  return (
    <Group title="Typography">
      <div className="flex flex-col gap-control-inset">
        {typographyTokens.map(({ className, token, variant }) => (
          <div className="flex items-baseline gap-control-inset" key={token}>
            <Text className={className} variant={variant}>
              The quick brown fox
            </Text>
            <Text variant="footnote">{token}</Text>
          </div>
        ))}
      </div>
    </Group>
  );
}

function Colors() {
  return (
    <Group title="Colors">
      <div className="grid grid-cols-5 gap-section">
        {colorTokens.map((token) => (
          <div className="flex items-center gap-control" key={token}>
            <div className={`size-8 shrink-0 rounded-control ${token}`} />
            <Text variant="footnote">{token}</Text>
          </div>
        ))}
      </div>
      <div className="flex flex-col gap-tight">
        {contentForegroundTokens.map((token) => (
          <Text className={token} key={token}>
            The quick brown fox jumps over the lazy dog ({token})
          </Text>
        ))}
      </div>
    </Group>
  );
}

function Fills() {
  return (
    <Group title="Fills">
      <div className="flex flex-col gap-tight">
        {fillTokens.map((token) => (
          <div
            className={`flex items-center px-control-inset py-control ${token}`}
            key={token}
          >
            <Text variant="footnote">{token}</Text>
          </div>
        ))}
      </div>
    </Group>
  );
}

function Spacing() {
  return (
    <Group title="Spacing">
      <div className="flex flex-col gap-control">
        {spacingTokens.map((token) => (
          <div className="flex items-center gap-control-inset" key={token}>
            <div className={`h-control-inset bg-fill ${token}`} />
            <Text variant="footnote">{token}</Text>
          </div>
        ))}
      </div>
    </Group>
  );
}

function Radii() {
  return (
    <Group title="Radii">
      <div className="flex flex-wrap gap-section">
        {radiusTokens.map((token) => (
          <div className="flex flex-col items-center gap-control" key={token}>
            <div className={`size-16 bg-fill ${token}`} />
            <Text variant="footnote">{token}</Text>
          </div>
        ))}
      </div>
    </Group>
  );
}

function Focus() {
  return (
    <Group title="Focus">
      <div className="flex flex-col items-center gap-control self-start">
        <div
          className={`h-control-height w-16 rounded-control bg-fill ring-3 ${focusStyles}`}
        />
        <Text variant="footnote">ring-focus-ring</Text>
      </div>
    </Group>
  );
}

function Icons() {
  return (
    <Group title="Icons">
      <div className="flex flex-wrap items-end gap-section">
        {iconTokens.map((token) => (
          <div className="flex flex-col items-center gap-control" key={token}>
            <Settings className={`text-content-fg ${token}`} />
            <Text variant="footnote">{token}</Text>
          </div>
        ))}
      </div>
    </Group>
  );
}

const meta = {
  parameters: {
    controls: { disable: true },
    layout: "fullscreen",
  },
  title: "Foundations/Tokens",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const All: Story = {
  render: () => (
    <Page>
      <Text as="h1" variant="title">
        Design tokens
      </Text>
      <Typography />
      <Colors />
      <Fills />
      <Spacing />
      <Radii />
      <Focus />
      <Icons />
    </Page>
  ),
};

export const TypographyTokens: Story = {
  render: () => (
    <Page>
      <Typography />
    </Page>
  ),
};

export const ColorTokens: Story = {
  render: () => (
    <Page>
      <Colors />
    </Page>
  ),
};

export const FillTokens: Story = {
  render: () => (
    <Page>
      <Fills />
    </Page>
  ),
};

export const SpacingTokens: Story = {
  render: () => (
    <Page>
      <Spacing />
    </Page>
  ),
};

export const RadiusTokens: Story = {
  render: () => (
    <Page>
      <Radii />
    </Page>
  ),
};

export const FocusTokens: Story = {
  render: () => (
    <Page>
      <Focus />
    </Page>
  ),
};

export const IconTokens: Story = {
  render: () => (
    <Page>
      <Icons />
    </Page>
  ),
};
