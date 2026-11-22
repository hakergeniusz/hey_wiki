// A typed character on the wiki screen, before the screen takes it: the
// search box holds the query, so a printable character lands in it and goes
// no further (away from the box — the results, the page — it is dropped).
// Control characters never arrive here: the engine sends those as keys,
// which the screen's key helper answers.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_screen_char(
    game: &crate::game::Game<'_>,
    screen: &ScreenView<'_>,
    c: char,
    modifiers: u32,
) -> core::ops::ControlFlow<()> {
        let _ = modifiers;
    if screen.id != "mod" || game.native_mod_screen.as_deref() != Some(wiki::ID) {
        return core::ops::ControlFlow::Continue(());
    }
    if !wiki::setting_on(wiki::file_setting) {
        return core::ops::ControlFlow::Continue(());
    }
    // The box takes what it holds the keyboard for; the screen keeps the
    // character either way, so nothing types through into the game.
    wiki::with_search(|view| view.insert(c));
    core::ops::ControlFlow::Break(())
}
