// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn, focusStyles } from "../lib/styling";

import { StoryGrid, StoryGridRow } from "./story-grid";

// Every design token from `src/styles`, rendered through the Tailwind
// utility generated from it, so a story fails visibly if a token is renamed
// or dropped. Each sits on the story grid, its utility class as the row's
// label, as the primitives' stories lay out their values.

const typographyTokens = [
  "text-title",
  "text-heading",
  "text-body",
  "text-label",
  "text-caption",
];

const textColorTokens = [
  "text-fg",
  "text-fg-secondary",
  "text-fg-tertiary",
  "text-fg-disabled",
];

const colorTokens = [
  "bg-fill",
  "bg-fill-strong",
  "bg-recess",
  "bg-divider",
  "bg-surface",
  "bg-primary",
  "bg-primary-hover",
  "bg-primary-pressed",
  "bg-on-primary",
  "bg-accent",
  "bg-on-accent",
  "bg-selected",
  "bg-emphasis",
  "bg-focus",
  "bg-success",
  "bg-info",
  "bg-warning",
  "bg-error",
];

// The raised material, at rest, hovered and pressed, and the primary
// action's fill with its bevel.
const materialTokens = [
  "control-raised",
  "control-raised control-raised-hover",
  "control-raised-pressed",
  "bg-primary primary-raised",
  "bg-primary-pressed primary-raised-pressed",
];

const radiusTokens = ["rounded-sm", "rounded-control", "rounded-panel"];

function TokenGrid({
  sample,
  tokens,
}: {
  sample: (token: string) => React.ReactNode;
  tokens: string[];
}) {
  return (
    <StoryGrid>
      {tokens.map((token) => (
        <StoryGridRow key={token} label={token}>
          {sample(token)}
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

export function Typography() {
  return (
    <TokenGrid
      sample={(token) => <span className={token}>The quick brown fox</span>}
      tokens={typographyTokens}
    />
  );
}

// Text colours read as text and the rest as swatches, on one grid: they are
// all colour tokens. Each swatch takes a hairline edge in the divider
// colour, so the ones close to the canvas (`surface`, `recess`, the colours
// that sit on a fill) still show as swatches rather than gaps.
export function Colors() {
  return (
    <TokenGrid
      sample={(token) =>
        token.startsWith("text-") ? (
          <span className={token}>The quick brown fox</span>
        ) : (
          <span
            className={cn(
              "block size-5 rounded-sm shadow-[inset_0_0_0_0.5px_var(--color-divider)]",
              token,
            )}
          />
        )
      }
      tokens={[...textColorTokens, ...colorTokens]}
    />
  );
}

export function Materials() {
  return (
    <TokenGrid
      sample={(token) => (
        <span
          className={cn("block h-control-height w-24 rounded-control", token)}
        />
      )}
      tokens={materialTokens}
    />
  );
}

export function Sizes() {
  return (
    <StoryGrid>
      <StoryGridRow label="h-control-height">
        <span className="block h-control-height w-16 rounded-control bg-fill-strong" />
      </StoryGridRow>
    </StoryGrid>
  );
}

// Each spacing shown as the bare space it measures, labelled with the
// utility it is used through.
const spacingTokens = [
  { label: "gap-inline", width: "w-inline" },
  { label: "px-inset", width: "w-inset" },
  { label: "ps-traffic-lights", width: "w-traffic-lights" },
];

export function Spacing() {
  return (
    <StoryGrid>
      {spacingTokens.map(({ label, width }) => (
        <StoryGridRow key={label} label={label}>
          <span className={cn("block h-4 rounded-sm bg-fill-strong", width)} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

export function Radii() {
  return (
    <TokenGrid
      sample={(token) => (
        <span className={cn("block size-10 bg-fill-strong", token)} />
      )}
      tokens={radiusTokens}
    />
  );
}

export function Focus() {
  return (
    <TokenGrid
      sample={() => (
        <span
          className={cn(
            "block h-control-height w-16 rounded-control bg-fill ring-3",
            focusStyles,
          )}
        />
      )}
      tokens={["ring-focus"]}
    />
  );
}
