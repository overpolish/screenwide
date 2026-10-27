// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ALargeSmall,
  Asterisk,
  Grid3x3,
  LayoutTemplate,
  Moon,
  Scaling,
  Sparkles,
  Sun,
  WholeWord,
} from "lucide-react";

import { type CaptionProfanity } from "../../../../bindings/CaptionProfanity";
import { type KeyboardEffectAnimation } from "../../../../bindings/KeyboardEffectAnimation";
import { type OverlaySize } from "../../../../bindings/OverlaySize";
import { type TextSize } from "../../../../bindings/TextSize";
import { CAPTION_WORDS } from "../../../../features/editor/recording/captions/recording-captions";
import { placementOptions } from "../../../../features/editor/tool-panels/placement-options";
import { NumberField } from "../../../../primitives/input-fields/number-field";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { PositionGrid } from "../../../../primitives/position-grid/position-grid";
import { type Editor } from "../model/use-editor";

import { CheckList, SectionLine } from "./island-parts";
import { type OverlayPart } from "./overlay-parts";
import { TemplateChoice } from "./template-choice";

// The captions' and the shortcuts' options, in the groups the app's panels
// keep them in, each opened from its icon in the island's row.

const label = (text: string) => (
  <span className="text-label tabular-nums">{text}</span>
);

const TEXT_SIZES: { id: TextSize; name: string; short: string }[] = [
  { id: "small", name: "Small", short: "S" },
  { id: "medium", name: "Medium", short: "M" },
  { id: "large", name: "Large", short: "L" },
  { id: "extra-large", name: "Extra large", short: "XL" },
];

const PROFANITY: { id: CaptionProfanity | null; name: string }[] = [
  { id: null, name: "As said" },
  { id: "mask", name: "Partly" },
  { id: "hide", name: "Fully" },
  { id: "remove", name: "Hidden" },
];

const KEY_SIZES: { id: OverlaySize; name: string }[] = [
  { id: "standard", name: "100%" },
  { id: "large", name: "150%" },
  { id: "extra-large", name: "200%" },
  { id: "huge", name: "300%" },
  { id: "giant", name: "500%" },
];

const ANIMATIONS: { id: KeyboardEffectAnimation; name: string }[] = [
  { id: "none", name: "None" },
  { id: "fade", name: "Fade" },
  { id: "pop", name: "Pop" },
];

export function captionParts(editor: Editor): OverlayPart[] {
  const { captions, setCaptions } = editor;
  const words = [
    { id: "holdUntilNext", label: "Hold until next caption" },
    { id: "hidePunctuation", label: "Hide punctuation" },
    { id: "hideFillers", label: "Hide filler words" },
  ] as const;
  return [
    {
      icon: LayoutTemplate,
      id: "template",
      label: "Template",
      node: (
        <TemplateChoice
          onChange={(template) => {
            setCaptions({ template });
          }}
          value={captions.template}
        />
      ),
    },
    {
      icon: WholeWord,
      id: "words",
      label: "Words",
      node: (
        <div className="w-56">
          <CheckList
            aria-label="Words"
            items={[...words]}
            onToggle={(id) => {
              const key = id as (typeof words)[number]["id"];
              setCaptions({ [key]: !captions[key] });
            }}
            selected={words
              .filter(({ id }) => captions[id])
              .map(({ id }) => id)}
          />
        </div>
      ),
    },
    {
      icon: ALargeSmall,
      id: "layout",
      label: "Layout",
      node: (
        <div className="flex w-56 flex-col gap-2">
          <SectionLine label="Size">
            <PillGroup
              aria-label="Size"
              items={TEXT_SIZES.map(({ id, name, short }) => ({
                ariaLabel: name,
                icon: label(short),
                id,
              }))}
              onSelectionChange={(size) => {
                setCaptions({ size: size as TextSize });
              }}
              selected={captions.size}
            />
          </SectionLine>
          <SectionLine label="Max words">
            <NumberField
              aria-label="Max words"
              className="w-16"
              maxValue={CAPTION_WORDS.maximum}
              minValue={CAPTION_WORDS.minimum}
              onChange={(maxWords) => {
                setCaptions({ maxWords });
              }}
              step={1}
              value={captions.maxWords}
            />
          </SectionLine>
          <SectionLine label="Lines">
            <PillGroup
              aria-label="Lines"
              items={[
                { ariaLabel: "One", icon: label("1"), id: "one" },
                { ariaLabel: "Two", icon: label("2"), id: "two" },
              ]}
              onSelectionChange={(lines) => {
                setCaptions({ lines: lines as "one" | "two" });
              }}
              selected={captions.lines}
            />
          </SectionLine>
          <CheckList
            aria-label="Case"
            items={[{ id: "allCaps", label: "All caps" }]}
            onToggle={() => {
              setCaptions({ allCaps: !captions.allCaps });
            }}
            selected={captions.allCaps ? ["allCaps"] : []}
          />
        </div>
      ),
    },
    {
      icon: Asterisk,
      id: "profanity",
      label: "Profanity",
      node: (
        <div className="w-48">
          <CheckList
            aria-label="Profanity"
            items={PROFANITY.map(({ id, name }) => ({
              id: id ?? "said",
              label: name,
            }))}
            onToggle={(id) => {
              setCaptions({
                profanity: id === "said" ? null : (id as CaptionProfanity),
              });
            }}
            selected={[captions.profanity ?? "said"]}
          />
        </div>
      ),
    },
    {
      icon: Grid3x3,
      id: "position",
      label: "Position",
      node: (
        <PositionGrid
          aria-label="Caption position"
          items={placementOptions()}
          onChange={(position) => {
            setCaptions({ position });
          }}
          value={captions.position}
        />
      ),
    },
  ];
}

export function keysParts(editor: Editor): OverlayPart[] {
  const { keys, setKeys } = editor;
  return [
    {
      icon: keys.appearance === "dark" ? Moon : Sun,
      id: "appearance",
      label: "Appearance",
      node: (
        <SectionLine label="Appearance">
          <PillGroup
            aria-label="Appearance"
            items={[
              {
                ariaLabel: "Light",
                icon: <Sun aria-hidden className="size-icon" />,
                id: "light",
              },
              {
                ariaLabel: "Dark",
                icon: <Moon aria-hidden className="size-icon" />,
                id: "dark",
              },
            ]}
            onSelectionChange={(appearance) => {
              setKeys({ appearance: appearance as "dark" | "light" });
            }}
            selected={keys.appearance}
          />
        </SectionLine>
      ),
    },
    {
      icon: Sparkles,
      id: "animation",
      label: "Animation",
      node: (
        <div className="w-40">
          <CheckList
            aria-label="Animation"
            items={ANIMATIONS.map(({ id, name }) => ({ id, label: name }))}
            onToggle={(id) => {
              setKeys({ animation: id as KeyboardEffectAnimation });
            }}
            selected={[keys.animation]}
          />
        </div>
      ),
    },
    {
      icon: Scaling,
      id: "size",
      label: "Size",
      node: (
        <div className="w-48">
          <CheckList
            aria-label="Size"
            items={KEY_SIZES.map(({ id, name }) => ({ id, label: name }))}
            onToggle={(id) => {
              setKeys({ size: id as OverlaySize });
            }}
            selected={[keys.size]}
          />
        </div>
      ),
    },
    {
      icon: Grid3x3,
      id: "position",
      label: "Position",
      node: (
        <PositionGrid
          aria-label="Shortcut position"
          items={placementOptions()}
          onChange={(position) => {
            setKeys({ position });
          }}
          value={keys.position}
        />
      ),
    },
  ];
}
