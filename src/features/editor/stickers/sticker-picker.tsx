// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ImagePlus } from "lucide-react";
import {
  memo,
  RefObject,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { Button } from "../../../components/base/button/button";
import { TextField } from "../../../components/base/input-fields/text-field";
import { ScrollArea } from "../../../components/base/scroll-area/scroll-area";
import { Text } from "../../../components/base/text/text";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

import {
  emojiAsset,
  loadStickerEmoji,
  matchingStickerEmoji,
  StickerEmojiGroup,
} from "./sticker-library";

import type { StickerArt } from "../annotations/annotations";

/**
 * The pictures a sticker can show, and the one it shows now: the emoji in
 * Unicode's groups, drawn by the system's own emoji font as the sticker will
 * be, narrowed by a search over their names and keywords, or a picture file
 * chosen with the Image button. Choosing one gives the chosen sticker that
 * picture, or the next sticker when nothing is chosen. A tile wears the
 * accent ring the app draws selection with, as a background swatch does.
 */
export function StickerPicker({
  isDisabled = false,
  onChange,
  onPickImage,
  value,
}: {
  onChange: (art: StickerArt) => void;
  /** Asks for a picture file and keeps it; null when none was chosen. */
  onPickImage: () => Promise<StickerArt | null>;
  value: StickerArt;
  isDisabled?: boolean;
}) {
  const [groups, setGroups] = useState<StickerEmojiGroup[] | null>(null);
  const [query, setQuery] = useState("");
  useEffect(() => {
    let disposed = false;
    void loadStickerEmoji().then((loaded) => {
      if (!disposed) setGroups(loaded);
    });
    return () => {
      disposed = true;
    };
  }, []);
  const shown = useMemo(
    () => (groups === null ? [] : matchingStickerEmoji(groups, query)),
    [groups, query],
  );
  // The panel renders on every step of a slider or a dial beside this, and
  // the grid is the most of what it would render: it is drawn again only
  // when what it shows changes, choosing through the latest handler.
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  // A choice is shown at once rather than after the editor has taken it and
  // sent the panel its picture back; it stands until that picture changes.
  const [pending, setPending] = useState<{ asset: string; over: string }>();
  const valueRef = useRef(value.asset);
  valueRef.current = value.asset;
  const chosen = pending?.over === value.asset ? pending.asset : value.asset;
  const choose = useCallback((asset: string) => {
    setPending({ asset, over: valueRef.current });
    onChangeRef.current({ aspect: 1, asset });
  }, []);
  // The picture in use is shown once the list arrives, however far down
  // its group lies.
  const chosenRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    chosenRef.current?.scrollIntoView({ block: "nearest" });
  }, [groups]);

  return (
    <div className="flex flex-col gap-control-inset">
      <div className="flex items-center gap-control">
        <TextField
          aria-label="Search stickers"
          className="min-w-0 flex-1"
          isDisabled={isDisabled}
          onChange={setQuery}
          placeholder="Search"
          value={query}
        />
        <Button
          aria-label="Choose a picture file"
          isDisabled={isDisabled}
          onPress={() => {
            onPickImage()
              .then((art) => {
                if (art) onChangeRef.current(art);
              })
              .catch((cause: unknown) => {
                console.error("Could not use that picture", cause);
              });
          }}
        >
          <ImagePlus aria-hidden="true" />
          Image…
        </Button>
      </div>
      <ScrollArea className="h-56">
        {groups !== null && shown.length === 0 ? (
          <Text variant="subheadline">No stickers match</Text>
        ) : (
          <EmojiGrid
            chosen={chosen}
            chosenRef={chosenRef}
            groups={shown}
            isDisabled={isDisabled}
            onChoose={choose}
          />
        )}
      </ScrollArea>
    </div>
  );
}

/** Every emoji shown, by group. The padding is room for the chosen tile's
 * ring, which stands off the tile, inside the scroll area's clip. */
const EmojiGrid = memo(function EmojiGrid({
  chosen,
  chosenRef,
  groups,
  isDisabled,
  onChoose,
}: {
  chosen: string;
  chosenRef: RefObject<HTMLButtonElement | null>;
  groups: StickerEmojiGroup[];
  isDisabled: boolean;
  onChoose: (asset: string) => void;
}) {
  const assets = useMemo(
    () =>
      groups.map(
        ({ emoji }) =>
          new Set(emoji.map(({ emoji: glyph }) => emojiAsset(glyph))),
      ),
    [groups],
  );
  return (
    <div className="flex flex-col gap-control-inset p-control">
      {groups.map(({ emoji, group }, index) => (
        <EmojiGroup
          chosen={assets[index]?.has(chosen) ? chosen : null}
          chosenRef={chosenRef}
          emoji={emoji}
          group={group}
          isDisabled={isDisabled}
          key={group}
          onChoose={onChoose}
        />
      ))}
    </div>
  );
});

/** One group's emoji. Every tile in a group reads its selection, so a choice
 * draws again only the groups it leaves and enters: each is told the chosen
 * picture only while it holds it. */
const EmojiGroup = memo(function EmojiGroup({
  chosen,
  chosenRef,
  emoji,
  group,
  isDisabled,
  onChoose,
}: {
  chosen: string | null;
  chosenRef: RefObject<HTMLButtonElement | null>;
  emoji: StickerEmojiGroup["emoji"];
  group: string;
  isDisabled: boolean;
  onChoose: (asset: string) => void;
}) {
  return (
    <section aria-label={group} className="flex flex-col gap-tight">
      <Text as="h3" variant="section">
        {group}
      </Text>
      <ToggleButtonGroup
        aria-label={group}
        className="flex flex-wrap gap-control"
        isDisabled={isDisabled}
        selectedKeys={chosen === null ? [] : [chosen]}
        selectionMode="single"
      >
        {emoji.map(({ emoji: glyph, name }) => {
          const asset = emojiAsset(glyph);
          return (
            <ToggleButton
              aria-label={name}
              className={cn(
                // An emoji is a picture rather than type, sized to fill its
                // tile, which no type size does.
                "flex size-icon-xl shrink-0 cursor-default items-center justify-center rounded-control text-[1.25rem] leading-none",
                focusStyles,
                elementFocusVisible,
                chosen === asset &&
                  "ring-2 ring-primary ring-offset-2 ring-offset-content",
                isDisabled && "opacity-50",
              )}
              id={asset}
              key={glyph}
              onPress={() => {
                onChoose(asset);
              }}
              ref={chosen === asset ? chosenRef : undefined}
            >
              <span aria-hidden="true">{glyph}</span>
            </ToggleButton>
          );
        })}
      </ToggleButtonGroup>
    </section>
  );
});
