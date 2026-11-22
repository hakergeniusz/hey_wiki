// A key on the wiki screen, before the screen acts on it: the mode answers
// (the box edits, Enter submits or opens, the results pick, the page
// scrolls), the URL a submit starts fetches natively — or, with the native
// view off, opens in the browser on the next tick. Held keys repeat through
// the events the engine sends; Escape is never the screen's, so the
// engine's own Escape still closes it.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_screen_key(
    game: &crate::game::Game<'_>,
    screen: &ScreenView<'_>,
    key: &str,
    repeat: bool,
    modifiers: u32,
) -> core::ops::ControlFlow<()> {
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
    /// A wiki search, fetched natively: the view waits for the ranked
    /// titles.
    fn search(query: &str) {
        let Some(url) = wiki::api_search_url(query) else {
            return;
        };
        wiki::with_viewer(|viewer| viewer.start(wiki::FetchKind::Search, query));
        wiki::fetch_async(wiki::FetchKind::Search, query.to_string(), url, http);
    }

    let _ = (repeat, modifiers);
    if screen.id != "mod" || game.native_mod_screen.as_deref() != Some(wiki::ID) {
        return core::ops::ControlFlow::Continue(());
    }
    if !wiki::setting_on(wiki::file_setting) {
        return core::ops::ControlFlow::Continue(());
    }
    let keyed = wiki::with_viewer(|viewer| viewer.key(wiki::search_key_of(key)));
    match keyed {
        wiki::Keyed::Nothing => core::ops::ControlFlow::Continue(()),
        wiki::Keyed::Kept => core::ops::ControlFlow::Break(()),
        wiki::Keyed::Submit(query) => {
            let native = wiki::native_view(|k| wiki::file_setting(k));
            match wiki::submit_plan(native, &query) {
                wiki::SubmitPlan::Native(query) => search(&query),
                wiki::SubmitPlan::Browser(url) => wiki::set_browser(url),
                wiki::SubmitPlan::Nothing => {}
            }
            core::ops::ControlFlow::Break(())
        }
        wiki::Keyed::Open(title) => {
            // The screen is already up (the results sent it): fetch alone.
            fetch_page(&title);
            core::ops::ControlFlow::Break(())
        }
    }
}
