# Hey Wiki

A [Horizon](https://github.com/horizon-rust/horizon) native mod (`.hnmod`) that puts the
[Minecraft Wiki](https://minecraft.wiki) a keypress away:

- **H** — open the wiki page of what you are looking at: the block or mob your crosshair
  targets (within the configurable reach), or — while a screen is open — the item under the
  cursor in the plain survival inventory.
- **Alt+H** — the wiki page of the item in your main hand.
- **B** — an in-game wiki search screen: type, Enter, and the results are listed to open.

The pages render **in the game** (a native screen: title, wrapped text, wheel/arrow
scrolling), fetched with the engine's own HTTP client on a worker thread. The browser stays
as the fallback: every page view offers **Open in browser** and **Copy link**, and with the
`native-view` setting off the mod falls back to the original's browser flow entirely — the
game's own open-or-copy link confirmation (Open / Copy link / Cancel) before any browser
opens. H and B rebind in Options > Mods, like any mod's keybinds.

Settings (Options > Mods > Hey Wiki): `enabled`, `native-view` (wiki in game, default on),
`confirm-open` (confirm before a browser opens, default on), `reach` (the crosshair's
lookup distance in blocks, default 5.2, click to cycle).

## Clean-room port

This is a from-scratch re-implementation of the
[Hey Wiki](https://modrinth.com/mod/hey-wiki) mod
([github.com/mc-wiki/minecraft-mod-heywiki](https://github.com/mc-wiki/minecraft-mod-heywiki),
**LGPL-3.0-only**) for Horizon's native mod system. No code was copied from the original:
the port follows the game's own seams (the crosshair pick, the mod-screen API, the engine's
link confirmation and HTTP client) and ships under the **MIT license**, which is
GPL-compatible. The original mod is credited as the concept's source; if you play modded
Java Minecraft, use theirs.

Deferred (not in this version): the `/wiki` and `/whatisthis` chat commands, `[[wiki
links]]` in chat, language override, and in-game browsing (MCBrowser/MCEF) — the native
page view covers the common case without an embedded browser.

## Building and verifying

The pinned Horizon revision is in `compatibility.json`; the tooling stages it, applies the
bundle and runs the game-side checks:

```sh
python3 tools/horizon.py check   # staged cargo check (compile gates)
python3 tools/horizon.py test    # staged runtime tests
python3 tools/horizon.py build   # staged game build
```

The runtime's own suite (format, clippy, unit tests) runs standalone:

```sh
python3 tools/test.py
```

`tools/ingame-test.txt` is the manual in-game checklist. `tools/package.py` builds the
deterministic `.hnmod`; the signature is made with `rsign` and checked against
`hey_wiki.pub` in the repo root.

## License

MIT — see [LICENSE](LICENSE). Concept and original implementation by the Hey Wiki
authors (LGPL-3.0-only, Modrinth project `6DnswkCZ`); this Horizon port is clean-room and
MIT-licensed, following the barrel_roll and elytra_slot precedent.
