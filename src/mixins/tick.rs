// Every tick: the mod's three keys poll here through their registered
// mappings — B opens the wiki screen, H looks up what the crosshair is on
// (the hovered slot's item while a screen is open), Alt+H the main hand's
// item — and the screen helper's deferrals happen where the &mut game is: a
// link for the clipboard, or a browser opening (the search box's Enter with
// the native view off, the page's Open in browser button). The browser goes
// where the confirmation setting sends it, the game's own link prompt or
// straight to the browser, and the screen closes first, so the prompt reads
// over the world as it always does.
//
// Playing means captured. The engine re-grabs on a screen close and on a
// refocus that lost a held pointer, but a browser round-trip hands the
// focus over while the pointer is already free — the link prompt needs the
// cursor — and nothing re-grabs on the way back. The tick re-asserts what
// playing means; the prompt dialog and its kin keep the cursor.
//
// The registry id of what the H press looks up, as the runtime's plan asks
// it: with a screen open, the plain survival inventory's hovered slot; over
// the world, the crosshair's block or mob within the reach setting. The
// hits the game already picked this tick (a guest's forced target, a hosted
// world's tick picks) stand as they are; otherwise the game's own rays run
// out to the setting — and the mob only when its hit is nearer than the
// block's, so a wall hides it as always.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_tick(game: &mut crate::game::Game<'_>) {
    // The registry id of what the crosshair is on, block or mob, at the
    // reach the setting holds.
    fn aimed_id(game: &mut crate::game::Game<'_>) -> Option<String> {
        let reach = f64::from(wiki::reach_of(wiki::file_setting));
        let (origin, dir) = game.aim()?;
        let block = game.forced_hit.or_else(|| match game.tick_hit.get() {
            Some(picked) => picked,
            None => game.clip_ray(origin, dir, reach),
        });
        let block_t = block.as_ref().map(|hit| {
            let [dx, dy, dz] = [
                hit.point[0] - origin[0],
                hit.point[1] - origin[1],
                hit.point[2] - origin[2],
            ];
            (dx * dx + dy * dy + dz * dz).sqrt()
        });
        let mob = match game.tick_mob {
            Some(picked) => picked.map(|(index, t, _)| (index, t)),
            None => game.mob_hit_within(reach).map(|(index, t, _)| (index, t)),
        };
        match wiki::resolve_target(block_t, mob) {
            Some(wiki::Which::Block) => {
                let hit = block?;
                let state = game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
                let name = game.res.facts().get(state).name.clone();
                Some(name.to_string())
            }
            Some(wiki::Which::Mob(index)) => {
                let kind = game.mobs.iter().nth(index)?.kind.clone();
                Some(kind.id.clone())
            }
            None => None,
        }
    }
    // The id of what the press looks up, per the focus the runtime picks.
    fn lookup_id(game: &mut crate::game::Game<'_>, alt: bool) -> Option<String> {
        // Where the press looks: a screen's hover, or the crosshair over the
        // world. `screen_view_now` is None over the world, Some over the
        // screens the hooks take — and the hover is the plain survival
        // inventory's only (a container's slots number its own grid, not the
        // player's).
        let focus = {
            let view = game.screen_view_now();
            wiki::lookup_focus(
                view.is_some(),
                view.is_some_and(|v| v.id == "inventory" && v.menu.is_none()),
                alt,
            )
        };
        match focus {
            wiki::Focus::Nothing => None,
            wiki::Focus::Held => game
                .inventory
                .get(game.selected)
                .and_then(|s| s.as_ref())
                .map(|s| s.item.to_string()),
            wiki::Focus::Hovered => {
                let slot = wiki::hover_slot(game.inventory_target().0?)?;
                match slot {
                    wiki::HoverSlot::Armor(which) => game
                        .armor
                        .get(which)?
                        .as_ref()
                        .map(|s| s.item.to_string()),
                    wiki::HoverSlot::Offhand => {
                        game.offhand.as_ref().map(|s| s.item.to_string())
                    }
                    wiki::HoverSlot::Inventory(which) => game
                        .inventory
                        .get(which)
                        .and_then(|s| s.as_ref())
                        .map(|s| s.item.to_string()),
                }
            }
            wiki::Focus::Crosshair => aimed_id(game),
        }
    }
    // The HTTP the workers run: the engine's own client, ten seconds to
    // answer, the body or nothing (any failure is the same nothing to the
    // fetch).
    fn http(url: &str) -> Option<String> {
        let response = ureq::get(url).timeout(std::time::Duration::from_secs(10)).call();
        response.ok().and_then(|answer| answer.into_string().ok())
    }
    /// A wiki page, fetched natively: the view starts waiting and the
    /// worker's answer fills it in when it lands.
    fn fetch_page(title: &str) {
        let Some(url) = wiki::api_extract_url(title) else {
            return;
        };
        wiki::with_viewer(|viewer| viewer.start(wiki::FetchKind::Page, title));
        wiki::fetch_async(wiki::FetchKind::Page, title.to_string(), url, http);
    }
    /// A wiki page, fetched natively and shown over the world: the mod's
    /// own screen opens on top of the fetch's wait.
    fn show_page(game: &mut crate::game::Game<'_>, title: &str) {
        fetch_page(title);
        game.mod_screen_open(wiki::ID);
    }
    /// Where a URL goes: behind the game's own open-or-copy link
    /// confirmation when the setting asks, else straight to the browser.
    fn open_url(game: &mut crate::game::Game<'_>, url: String) {
        let confirm = wiki::confirm_open(wiki::file_setting);
        match wiki::open_plan(confirm) {
            wiki::OpenPlan::Prompt => {
                // The prompt is answered with the cursor, as vanilla's
                // always is. The H press finds the pointer held; let it go
                // first, or the dialog's buttons stay out of reach while the
                // view turns under the mouse. The tick re-grabs once the
                // prompt is answered no, the engine's own refocus re-grab
                // after the browser round-trip on yes.
                if game.captured {
                    game.capture(false);
                }
                game.link_prompt = Some(url);
            }
            wiki::OpenPlan::Direct => crate::game::packs::open_uri(&url),
        }
    }

    // The keys first: B opens the wiki screen, H looks the target up,
    // Alt+H the held item — each a registered mapping the tick polls.
    let enabled = wiki::setting_on(wiki::file_setting);
    let on_mod = game.native_mod_screen.as_deref() == Some(wiki::ID);
    let playing = game.playing();
    for (mapping, key, alt) in [
        (wiki::OPEN_KEYBIND, wiki::OPEN_KEYBIND, false),
        (wiki::SEARCH_KEYBIND, wiki::SEARCH_KEYBIND, false),
        ("held", wiki::SEARCH_KEYBIND, true),
    ] {
        if !game.consume_click(wiki::ID, mapping) {
            continue;
        }
        let press = wiki::route(true, enabled, playing, on_mod, key, "open", "search", alt);
        match press {
            wiki::Press::None => {}
            wiki::Press::Search => {
                wiki::reset_screen();
                game.mod_screen_open(wiki::ID);
            }
            wiki::Press::Held | wiki::Press::Lookup => {
                // The id of the answer: Alt+H is the hand's item; H is the
                // crosshair's target (or the hovered slot over a screen).
                let id = if press == wiki::Press::Held {
                    game.inventory
                        .get(game.selected)
                        .and_then(|s| s.as_ref())
                        .map(|s| s.item.to_string())
                } else {
                    lookup_id(game, false)
                };
                // What it becomes, as the native-view setting sends it: the
                // page fetched into the mod's own screen, or the browser
                // behind the game's link confirmation.
                let native = wiki::native_view(wiki::file_setting);
                match wiki::h_plan(native, id) {
                    wiki::HPlan::Native(title) => show_page(game, &title),
                    wiki::HPlan::Browser(url) => open_url(game, url),
                    wiki::HPlan::Nothing => {}
                }
            }
        }
    }

    // The screen helper's deferrals, where the &mut game is.
    if wiki::should_regrab(
        game.screen == crate::game::Screen::Playing,
        game.focused,
        game.keyboard_mouse,
        game.captured,
        game.screen_over_playing(),
    ) {
        game.capture(true);
    }
    if let Some(url) = wiki::take_copy() {
        use horizon_client::clipboard::Clipboard;
        game.clipboard.set(&url);
    }
    let Some(url) = wiki::take_browser() else {
        return;
    };
    if game.native_mod_screen.as_deref() == Some(wiki::ID) {
        game.close_mod_screen();
    }
    let confirm = wiki::confirm_open(wiki::file_setting);
    match wiki::open_plan(confirm) {
        wiki::OpenPlan::Prompt => {
            if game.captured {
                game.capture(false);
            }
            game.link_prompt = Some(url);
        }
        wiki::OpenPlan::Direct => crate::game::packs::open_uri(&url),
    }
}
