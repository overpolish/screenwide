// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

// The licence server for Screenwide's paid tools: it checks a key with
// Payhip and signs a record the app verifies offline
// (`src-tauri/crates/licensing`). It keeps no state: Payhip is the record of
// which keys exist and which are disabled.

import { activate } from "./activate";

export default {
  async fetch(request, env): Promise<Response> {
    const { pathname } = new URL(request.url);
    if (request.method !== "POST" || pathname !== "/activate") {
      return new Response("Not found", { status: 404 });
    }
    // Wrapped so `fetch` is always called as the global's own.
    return activate(request, env, (input, init) => fetch(input, init));
  },
} satisfies ExportedHandler<Env>;
