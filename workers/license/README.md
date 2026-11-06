<!--
SPDX-FileCopyrightText: 2026 overpolish
SPDX-License-Identifier: Elastic-2.0
-->

# Licence server

The Cloudflare Worker at `license.getscreenwide.com` that the app asks about a licence key (`src-tauri/crates/licensing`). It checks the key with Payhip against every tool's product and signs a record of the tools the key grants, which the app verifies offline. It keeps no state: Payhip is the record of which keys exist and which are disabled.

## Answers

`POST /activate` with `{ "licenseKey": "…", "installationId": "…" }`:

| Answer | When |
| --- | --- |
| `200 { payload, signature }` | A product knows the key and it is enabled |
| `403 { "error": "disabled" }` | A product knows the key but Payhip has disabled it, as on a refund |
| `400 { "error": "invalid_key" }` | Every product clearly does not know the key |
| `502 { "error": "upstream" }` | Payhip could not answer for a product, and none granted the key |
| `400 { "error": "bad_request" }` | No key or installation ID in the body |
| `500 { "error": "not_configured" }` | A secret is missing |

The app takes only `disabled` and `invalid_key` as a no; everything else reads as no answer, and the app keeps the key. A key is "not known" only on a clear answer from Payhip: its reply for a key a product does not have, a 400 of `{"data":[],"error":true}`, or a 404 or a success without a licence. Payhip answers a product secret it rejects with a 401 (`authentication_fail`), and that counts as Payhip failing, as does any other error, so a mistyped or rotated secret cannot take keys away from buyers at their next check.

## Secrets

Set each with `pnpm wrangler secret put <NAME>` from this folder, entered at the prompt or piped from a file, never typed into a command:

- `LICENSE_SIGNING_KEY`: the private half of the key pair the app checks records against, PKCS#8 in base64. Its public half is `PUBLIC_KEY` in `src-tauri/crates/licensing/src/server.rs`.
- `PAYHIP_SECRET_RULER`, `PAYHIP_SECRET_REFERENCES`, `PAYHIP_SECRET_GLIDE`, `PAYHIP_SECRET_ANNOTATE`: each product's secret key, from the bottom of its edit page on Payhip once licence keys are switched on.

## Work on it

- `pnpm --filter screenwide-license-worker typecheck` writes the Worker's types (`wrangler types`) and checks them.
- `pnpm exec vitest run workers/license` runs its tests, with Payhip faked.
- `pnpm --filter screenwide-license-worker dev` runs it locally; it reads secrets from a `.dev.vars` file here, which git ignores.
- `pnpm --filter screenwide-license-worker deploy` publishes it, after `pnpm wrangler login`. The custom domain needs the `getscreenwide.com` zone on the same Cloudflare account.
