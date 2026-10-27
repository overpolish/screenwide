// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ALargeSmall,
  Asterisk,
  Grid3x3,
  LayoutTemplate,
  Minus,
  Moon,
  Plus,
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
import { IslandChoice } from "../../../../primitives/island/island-choice";
import { IslandButton } from "../../../../primitives/island/island-controls";
import { type Editor } from "../model/use-editor";

import { CheckRow, SectionLine } from "./island-parts";
import { type OverlayPart } from "./overlay-parts";
import { PlacementChoice } from "./placement-choice";
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
  const words = (
    [
      ["holdUntilNext", "Hold until next caption"],
      ["hidePunctuation", "Hide punctuation"],
      ["hideFillers", "Hide filler words"],
    ] as const
  ).map(([key, name]) => (
    <CheckRow
      isChosen={captions[key]}
      key={key}
      onPress={() => {
        setCaptions({ [key]: !captions[key] });
      }}
    >
      {name}
    </CheckRow>
  ));
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
      node: <div className="flex w-56 flex-col">{words}</div>,
    },
    {
      icon: ALargeSmall,
      id: "layout",
      label: "Layout",
      node: (
        <div className="flex w-56 flex-col gap-2">
          <SectionLine label="Size">
            <IslandChoice
              items={TEXT_SIZES.map(({ id, name, short }) => ({
                content: label(short),
                id,
                label: name,
              }))}
              label="Size"
              onChange={(size) => {
                setCaptions({ size });
              }}
              value={captions.size}
            />
          </SectionLine>
          <SectionLine label="Max words">
            <IslandButton
              aria-label="Fewer words"
              isDisabled={captions.maxWords <= CAPTION_WORDS.minimum}
              onPress={() => {
                setCaptions({ maxWords: captions.maxWords - 1 });
              }}
            >
              <Minus aria-hidden className="size-icon" />
            </IslandButton>
            <span className="w-6 text-center text-label tabular-nums">
              {captions.maxWords}
            </span>
            <IslandButton
              aria-label="More words"
              isDisabled={captions.maxWords >= CAPTION_WORDS.maximum}
              onPress={() => {
                setCaptions({ maxWords: captions.maxWords + 1 });
              }}
            >
              <Plus aria-hidden className="size-icon" />
            </IslandButton>
          </SectionLine>
          <SectionLine label="Lines">
            <IslandChoice
              items={[
                { content: label("1"), id: "one" as const, label: "One" },
                { content: label("2"), id: "two" as const, label: "Two" },
              ]}
              label="Lines"
              onChange={(lines) => {
                setCaptions({ lines });
              }}
              value={captions.lines}
            />
          </SectionLine>
          <CheckRow
            isChosen={captions.allCaps}
            onPress={() => {
              setCaptions({ allCaps: !captions.allCaps });
            }}
          >
            All caps
          </CheckRow>
        </div>
      ),
    },
    {
      icon: Asterisk,
      id: "profanity",
      label: "Profanity",
      node: (
        <div className="flex w-48 flex-col">
          {PROFANITY.map(({ id, name }) => (
            <CheckRow
              isChosen={captions.profanity === id}
              key={name}
              onPress={() => {
                setCaptions({ profanity: id });
              }}
            >
              {name}
            </CheckRow>
          ))}
        </div>
      ),
    },
    {
      icon: Grid3x3,
      id: "position",
      label: "Position",
      node: (
        <PlacementChoice
          label="Caption position"
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
          <IslandChoice
            items={[
              {
                content: <Sun aria-hidden className="size-icon" />,
                id: "light" as const,
                label: "Light",
              },
              {
                content: <Moon aria-hidden className="size-icon" />,
                id: "dark" as const,
                label: "Dark",
              },
            ]}
            label="Appearance"
            onChange={(appearance) => {
              setKeys({ appearance });
            }}
            value={keys.appearance}
          />
        </SectionLine>
      ),
    },
    {
      icon: Sparkles,
      id: "animation",
      label: "Animation",
      node: (
        <div className="flex w-40 flex-col">
          {ANIMATIONS.map(({ id, name }) => (
            <CheckRow
              isChosen={keys.animation === id}
              key={id}
              onPress={() => {
                setKeys({ animation: id });
              }}
            >
              {name}
            </CheckRow>
          ))}
        </div>
      ),
    },
    {
      icon: Scaling,
      id: "size",
      label: "Size",
      node: (
        <div className="flex w-48 flex-col">
          {KEY_SIZES.map(({ id, name }) => (
            <CheckRow
              isChosen={keys.size === id}
              key={id}
              onPress={() => {
                setKeys({ size: id });
              }}
            >
              {name}
            </CheckRow>
          ))}
        </div>
      ),
    },
    {
      icon: Grid3x3,
      id: "position",
      label: "Position",
      node: (
        <PlacementChoice
          label="Shortcut position"
          onChange={(position) => {
            setKeys({ position });
          }}
          value={keys.position}
        />
      ),
    },
  ];
}
