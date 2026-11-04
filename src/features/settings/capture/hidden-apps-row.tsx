// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { AppWindow, EyeOff } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useCollator } from "react-aria";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { Text } from "../../../primitives/text/text";
import { TooltipTrigger } from "../../../primitives/tooltip/tooltip-trigger";
import { type RunningApp } from "../api";
import { hiddenAppsRow } from "../sections/general-rows";
import { useSettingsApi } from "../settings-api-context";

import type { HiddenApp } from "../../../bindings/HiddenApp";

// The apps kept out of captures, chosen in place: every running app
// Screenwide can hide, each its icon, pressed to hide it or let it back,
// with any hidden app not running now among them, so it can be let back
// without starting it. A hidden app's icon is set back with a mark in its
// corner, so the row says at a glance what is left out. In name order, so a
// tile stays put as it is pressed. The apps are asked for again as the
// window comes back into focus, for apps opened meanwhile.

/** A tile's side, round an app icon a little smaller, in pixels. */
const TILE_PX = 40;
const ICON_PX = 30;

type Offered = { iconPath: string | null; id: string; name: string };

export function HiddenAppsRow({
  apps,
  isDisabled,
  onChange,
}: {
  apps: HiddenApp[];
  isDisabled: boolean;
  onChange: (apps: HiddenApp[]) => void;
}) {
  const { listRunningApps } = useSettingsApi();
  const listRef = useRef(listRunningApps);
  listRef.current = listRunningApps;
  const [running, setRunning] = useState<RunningApp[]>([]);
  const collator = useCollator();
  const item = hiddenAppsRow();

  useEffect(() => {
    let isCurrent = true;
    const load = () => {
      listRef
        .current()
        .then((next) => {
          if (isCurrent) setRunning(next);
        })
        .catch((cause: unknown) => {
          console.error("Could not list the running apps", cause);
        });
    };
    load();
    window.addEventListener("focus", load);
    return () => {
      isCurrent = false;
      window.removeEventListener("focus", load);
    };
  }, []);

  const offered: (Offered & { label: string })[] = [
    ...running.map((app) => ({ ...app, name: app.label })),
    ...apps
      .filter((app) => !running.some((runs) => runs.id === app.id))
      .map((app) => ({
        iconPath: null,
        id: app.id,
        label: app.label,
        name: t("settings-hidden-apps-not-running", { app: app.label }),
      })),
  ].sort((first, second) => collator.compare(first.label, second.label));

  return (
    <ControlRow
      className="flex-wrap"
      controlClassName="basis-full"
      description={item.description}
      title={item.title}
    >
      {(controlProps) =>
        offered.length === 0 ? (
          <Text variant="caption">{t("settings-hidden-apps-empty")}</Text>
        ) : (
          <ToggleButtonGroup
            {...controlProps}
            className="flex flex-wrap gap-inline"
            isDisabled={isDisabled}
            onSelectionChange={(keys) => {
              onChange(
                offered
                  .filter(({ id }) => keys.has(id))
                  .map(({ id, label }) => ({ id, label })),
              );
            }}
            selectedKeys={apps.map((app) => app.id)}
            selectionMode="multiple"
          >
            {offered.map((app) => (
              <TooltipTrigger key={app.id} tooltip={app.name}>
                <ToggleButton
                  aria-label={app.name}
                  className={cn(
                    "relative grid cursor-default place-items-center rounded-control transition-colors data-[hovered]:bg-fill",
                    focusStyles,
                    elementFocusVisible,
                  )}
                  id={app.id}
                  style={{ height: TILE_PX, width: TILE_PX }}
                >
                  {({ isSelected }) => (
                    <>
                      {app.iconPath ? (
                        <img
                          alt=""
                          className={cn(
                            "transition-opacity",
                            isSelected && "opacity-40",
                          )}
                          src={convertFileSrc(app.iconPath)}
                          style={{ height: ICON_PX, width: ICON_PX }}
                        />
                      ) : (
                        <AppWindow
                          aria-hidden
                          className={cn(
                            "size-icon-lg text-fg-secondary transition-opacity",
                            isSelected && "opacity-40",
                          )}
                        />
                      )}
                      {isSelected ? (
                        <span
                          aria-hidden
                          className="absolute end-0.5 bottom-0.5 grid size-mark place-items-center rounded-full bg-primary text-on-primary"
                        >
                          <EyeOff className="size-icon-xs" />
                        </span>
                      ) : null}
                    </>
                  )}
                </ToggleButton>
              </TooltipTrigger>
            ))}
          </ToggleButtonGroup>
        )
      }
    </ControlRow>
  );
}
