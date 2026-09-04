// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { X } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "../../components/base/button/button";
import { IconButton } from "../../components/base/button/icon-button";
import { Keyboard, Shortcut } from "../../components/base/keyboard/keyboard";

import { beginShortcutCapture, endShortcutCapture } from "./api";

const keyName = (code: string) => {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  return code.replace("Arrow", "");
};

const shortcutFromEvent = (event: KeyboardEvent) => {
  if (["Alt", "Control", "Meta", "Shift"].includes(event.key)) return null;
  const modifiers = [
    event.metaKey ? "Command" : null,
    event.ctrlKey ? "Control" : null,
    event.altKey ? "Alt" : null,
    event.shiftKey ? "Shift" : null,
  ].filter(Boolean);
  if (modifiers.length === 0) return null;
  return [...modifiers, event.code].join("+");
};

const displayShortcut = (shortcut: string | null) => {
  if (!shortcut) return [];
  const mac = navigator.userAgent.includes("Mac");
  return shortcut.split("+").map((part) => {
    if (part === "CommandOrControl") return mac ? "Command" : "Control";
    if (part === "Command" || part === "Super") return "Meta";
    if (part === "Control") return "Control";
    if (part === "Alt") return mac ? "⌥" : "Alt";
    if (part === "Shift") return "Shift";
    return keyName(part);
  });
};

export function ShortcutField({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (shortcut: string | null) => void;
  value: string | null;
  isDisabled?: boolean;
}) {
  const [listening, setListening] = useState(false);

  useEffect(() => {
    if (!listening) return;
    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.key === "Escape") {
        setListening(false);
        void endShortcutCapture();
        return;
      }
      if (event.key === "Backspace" || event.key === "Delete") {
        onChange(null);
        setListening(false);
        return;
      }
      const shortcut = shortcutFromEvent(event);
      if (!shortcut) return;
      onChange(shortcut);
      setListening(false);
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, [listening, onChange]);

  const keys = displayShortcut(value);

  return (
    <div className="flex items-center gap-1">
      <Button
        className="min-w-32 justify-center whitespace-nowrap"
        isDisabled={isDisabled}
        onPress={() => {
          void beginShortcutCapture().then(() => {
            setListening(true);
          });
        }}
        size="compact"
        variant="ghost"
      >
        {listening ? (
          <span className="text-xs text-muted">Press shortcut…</span>
        ) : keys.length > 0 ? (
          <Shortcut>
            {keys.map((key, index) => (
              <Keyboard key={`${key}-${index.toString()}`}>{key}</Keyboard>
            ))}
          </Shortcut>
        ) : (
          <span className="text-xs text-muted">Not set</span>
        )}
      </Button>
      <IconButton
        aria-label="Clear shortcut"
        isDisabled={isDisabled || value === null}
        onPress={() => {
          onChange(null);
        }}
        size="compact"
      >
        <X size={14} />
      </IconButton>
    </div>
  );
}
