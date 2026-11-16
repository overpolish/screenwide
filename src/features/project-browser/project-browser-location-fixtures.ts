// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ProjectLocation } from "./types";

/** The projects folder, a folder beside it, one on an external drive, and
 * one on a drive that is not connected, which the sidebar shows disabled. */
export const locations: ProjectLocation[] = [
  {
    available: true,
    isDefault: true,
    name: "Screenwide",
    onOtherDrive: false,
    path: "/Users/demo/Movies/Screenwide",
  },
  {
    available: true,
    isDefault: false,
    name: "Tutorials",
    onOtherDrive: false,
    path: "/Users/demo/Documents/Tutorials",
  },
  {
    available: true,
    isDefault: false,
    name: "Client work",
    onOtherDrive: true,
    path: "/Volumes/Archive/Client work",
  },
  {
    available: false,
    isDefault: false,
    name: "Old drive",
    onOtherDrive: true,
    path: "/Volumes/Old drive/Recordings",
  },
];
