<!--
SPDX-FileCopyrightText: 2026 overpolish
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Openverse search

The Cloudflare Worker at `openverse.getscreenwide.com` that the Library island searches Openverse through (`src-tauri/crates/openverse`). It holds the app's Openverse registration, so the client secret never ships in the app and no one can take it to use up the searches every user shares. It only ever answers with what may be used with no more than a credit: public domain, CC0 and CC BY, never ShareAlike, NonCommercial or NoDerivatives.

## Answers

`GET /search?kind=music|effect|image&q=…&page=1` (pages 1 to 12, 20 results each):

| Answer | When |
| --- | --- |
| `200 { found, pageCount }` | Openverse answered; `found` is cut down to what the app shows and credits |
| `400 { "error": "unknown_kind" \| "bad_query" \| "bad_page" }` | The search is not one the app makes |
| `429 { "error": "busy" }` | This address has searched 20 times in the last minute, or Openverse is rate limiting the app |
| `502 { "error": "upstream" }` | Openverse could not answer |
| `500 { "error": "not_configured" }` | A secret is missing |

Songs are searched as Openverse's `short` and `medium` lengths (30 seconds to 10 minutes), sound effects as `shortest` (under 30 seconds). The app's token, which lasts 12 hours, and every answer, for a day, are kept in the data centre's cache.

`GET /work?kind=music|effect|image&id=<Openverse id>` looks one work up again, which the app does as it downloads it, so a work taken down or moved to a licence the app may not use is not added, and its credit is as it stands now:

| Answer | When |
| --- | --- |
| `200 { found }` | `found` is the work as a search answers with it, or `null` where it is gone or no longer under a licence the app may use |
| `400 { "error": "unknown_kind" \| "bad_id" }` | The id is not an Openverse id |

It counts against the same 20 a minute, answers `429` and `502` as a search does, and is kept for an hour.

## Secrets

Set each with `pnpm wrangler secret put <NAME>` from this folder, entered at the prompt, never typed into a command:

- `OPENVERSE_CLIENT_ID` and `OPENVERSE_CLIENT_SECRET`: the app's registration with Openverse (`POST https://api.openverse.org/v1/auth_tokens/register/`), whose email has been confirmed.

## Work on it

- `pnpm --filter screenwide-openverse-worker typecheck` writes the Worker's types (`wrangler types`) and checks them.
- `pnpm exec vitest run workers/openverse` runs its tests, with Openverse faked.
- `pnpm --filter screenwide-openverse-worker dev` runs it locally; it reads the secrets from a `.dev.vars` file here, which git ignores. Start the app with `SCREENWIDE_OPENVERSE_URL=http://localhost:8787` to ask it rather than the published Worker.
- `pnpm --filter screenwide-openverse-worker deploy` publishes it, after `pnpm wrangler login`. The custom domain needs the `getscreenwide.com` zone on the same Cloudflare account.
