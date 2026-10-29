// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReplaceAll } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { TextField } from "../../../../primitives/input-fields/text-field";
import { IslandFill } from "../../../../primitives/island/island-fill";

/**
 * A Remove drawn small: what it takes as a glyph, struck through while it
 * is about to take it, and how many, in a quiet pill whose fill grows in
 * under the pointer as the island's controls' does. Islands show no
 * tooltips, so its full name ("Remove 6 pauses") is its accessible name
 * alone. While hovered or focused it previews what it would cut, so those
 * words show red before it is pressed; it goes once there is nothing left
 * to take, and its preview with it.
 */
export function ScriptRemoveChip({
  count,
  glyph,
  isDisabled,
  label,
  onPreview,
  onRemove,
}: {
  count: number;
  /** The glyph for what it takes, struck through while previewing. */
  glyph: (isPreviewing: boolean) => ReactNode;
  isDisabled: boolean;
  label: string;
  /** Told whether it is hovered or focused. */
  onPreview: (isPreviewing: boolean) => void;
  onRemove: () => void;
}) {
  const [isHovered, setIsHovered] = useState(false);
  const [isFocused, setIsFocused] = useState(false);
  const isPreviewing = (isHovered || isFocused) && !isDisabled;
  useEffect(() => {
    onPreview(isPreviewing);
  }, [isPreviewing, onPreview]);
  useEffect(
    () => () => {
      onPreview(false);
    },
    [onPreview],
  );
  return (
    <Button
      aria-label={label}
      className="group px-inset-sm text-fg-secondary enabled:hover:text-fg"
      isDisabled={isDisabled}
      onFocusChange={setIsFocused}
      onHoverChange={setIsHovered}
      onPress={onRemove}
      variant="ghost"
    >
      <IslandFill isChosen={false} />
      {glyph(isPreviewing)}
      <span aria-hidden className="tabular-nums">
        {count}
      </span>
    </Button>
  );
}

/** What every match of the search becomes, and the button that makes it
 * so, as one step of the edit history. */
export function ScriptReplaceLine({
  found,
  isDisabled,
  onReplaceAll,
}: {
  found: number;
  isDisabled: boolean;
  onReplaceAll: (replacement: string) => void;
}) {
  const [replacement, setReplacement] = useState("");
  return (
    <div className="flex items-center gap-separate">
      <TextField
        aria-label={t("editor-island-script-replace-with")}
        className="min-w-0 flex-1"
        icon={<ReplaceAll />}
        isClearable
        isDisabled={isDisabled}
        onChange={setReplacement}
        placeholder={t("editor-island-script-replace-with")}
        value={replacement}
      />
      <Button
        isDisabled={isDisabled || found === 0}
        onPress={() => {
          onReplaceAll(replacement);
        }}
      >
        {t("editor-island-script-replace-all")}
      </Button>
    </div>
  );
}
