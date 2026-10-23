# TLI-Sattatracker

Transparent always-on-top overlay that tracks loot, expenses and profit per map in Torchlight Infinite. It reads the game's log file, so it does not touch the game process.

Built with Tauri 2 (Rust backend) and Vue 3.

## Features

- Loot feed with prices, map timer, per-map and per-session statistics
- Map History: every finished map with time, profit, loot, expenses and a net-profit-over-time chart
- Sessions survive restarts; Reset Session archives the current one
- Map expense tracking (compasses, beacons and other consumables)
- Inventory snapshot and total value
- Optional auction tax and auction-trade filtering
- Draggable widgets, themes, low-stock warning

## Hotkeys

| Key | Action |
| --- | --- |
| F7 | Show / hide the overlay |
| F8 | Open the menu (settings, inventory, map history, layout editing) |

The game must have logging enabled: in game press ESC, then enable the log option, and open the inventory once so the tracker can sync.

## Build

Requirements: Node.js 20+, Rust stable, WebView2.

```
npm install
npm run tauri dev
npm run tauri build
```

The release build embeds a manifest that requests administrator rights. Debug builds do not.

## Prices

Prices come from the public ETor price API:

- `GET /api/prices-snapshot/{season}` returns `{ "data": [ { "id": "350513", "price": 910.8 } ] }`, the whole price table of a season in FE
- `GET /api/player/season-status` is used to find the current season when the one from the log has no data

Primary host `https://etor-zero.981001.xyz/etor-api`, fallback `https://etor-beta.710421059.xyz/etor-api`. The season id is read from the game log. The table is refreshed every 10 minutes and cached on disk, so the tracker keeps working with the last known prices if the service is unreachable. This project does not operate the API and has no control over its availability or data.

To use another source, set `TLI_PRICE_API_URL` to an `https://` base URL with the same response format. Plain `http://` is ignored.

All price data is treated as untrusted: HTTPS only, response size limit, item ids must be numeric, prices must be finite and within a sane range.

## Assets

Images and animations (item icons, the piggy bank widget) are not bundled with the app. On launch it checks the GitHub release tagged `Assets` of [sattatrapy/sattatrackerUpdate](https://github.com/sattatrapy/sattatrackerUpdate/releases/tag/Assets). When the release date changes it downloads `assets.zip` and `assets.zip.sig` and unpacks the archive into `public_assets` inside the app local data directory (`%LOCALAPPDATA%\com.fedin.sattatracker`).

The archive is accepted only if its minisign signature matches the public key built into `src-tauri/src/assets.rs`. It is then unpacked into a temporary directory with strict rules: only `icons/` and `piganim/`, only PNG/WebP files, safe file names, size and count limits. Only after everything passes the old directory is replaced. If the signature is missing or wrong, nothing is installed and the current assets stay.

To host your own assets, create a key with `npm run tauri signer generate`, replace `PUBLIC_KEY_B64` in `src-tauri/src/assets.rs` and the URLs next to it, and publish `assets.zip` together with the file produced by `npm run tauri signer sign assets.zip`.

Without network access on first launch the tracker works, but those images are missing.

## Security notes

- The webview runs with a strict Content Security Policy and cannot make network requests; only the Rust side talks to the network, over HTTPS.
- File access from the webview is limited to the downloaded assets directory.
- Everything read back from disk (saved session, price cache) is validated before use.
- Saved state accepts writes only from the main window and has a fixed path and a size limit.

## Data

Settings are stored in `config.json` next to the executable. The saved session and map history (`tracker_state.json`), the price cache and the assets are kept in the app local data directory.

## License

MIT
