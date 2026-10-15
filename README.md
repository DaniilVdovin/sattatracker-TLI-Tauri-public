# TLI-Sattatracker

Transparent always-on-top overlay that tracks loot, expenses and profit per map in Torchlight Infinite. It reads the game's log file, so it does not touch the game process.

Built with Tauri 2 (Rust backend) and Vue 3.

## Features

- Loot feed with prices, map timer, per-map and per-session statistics
- Map expense tracking (compasses, beacons and other consumables)
- Inventory snapshot and total value
- Optional auction tax and auction-trade filtering
- Draggable widgets, themes, low-stock warning

## Hotkeys

| Key | Action |
| --- | --- |
| F7 | Show / hide the overlay |
| F8 | Toggle edit mode (move widgets) |

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

By default prices come from `http://115.231.176.101:8080/get?season_id=N`, the same endpoint the first (Python) version of this tracker used. It is an external service that returns the price table by season with Chinese item names. This project does not operate it and has no control over its availability or data.

The season id is read from the game log. The last received table is cached on disk, so the tracker keeps working with the last known prices if the server is unreachable.

To use another source, set `TLI_PRICE_API_URL` to its base URL (`?season_id=N` is appended). The response format must be `{ "<item_id>": { "price": <number> } }`.

## Assets

Images and animations (for example the piggy bank widget) are not bundled with the app. On first launch it downloads them from the GitHub release tagged `Assets` of [sattatrapy/sattatrackerUpdate](https://github.com/sattatrapy/sattatrackerUpdate/releases/tag/Assets), file `assets.zip`, and unpacks them into `public_assets` inside the app local data directory (`%LOCALAPPDATA%\com.fedin.sattatracker`). The release `published_at` date is stored as the local version and the archive is downloaded again when it changes.

Without network access on first launch the tracker works, but those images are missing.

## Configuration

Settings are stored in `config.json` next to the executable. The price cache and assets are kept in the app local data directory.

## License

MIT
