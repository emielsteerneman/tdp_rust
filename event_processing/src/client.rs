use std::future::Future;

// Only the HTTP middleware sees the user agent, but events are dispatched from deep inside the
// `api` handlers. Rather than thread a flag through every handler, the middleware wraps the
// request in `scope`, and `EventDispatcher::dispatch` (which runs inside that request) reads it
// with `is_crawler` to skip listeners that don't want crawler events.
tokio::task_local! {
    static CRAWLER: bool;
}

/// Lowercase substrings that mark a self-identified crawler. `bot` covers Googlebot, bingbot,
/// Applebot, AhrefsBot, GPTBot, ClaudeBot, AdsBot-Google, quillbot, ...
const CRAWLER_MARKERS: &[&str] = &[
    "bot",
    "crawl",
    "spider",
    "slurp",
    "googleother",
    "bingpreview",
    "facebookexternalhit",
];

/// Whether the user agent self-identifies as a crawler. Scrapers posing as a regular browser
/// are not caught.
pub fn is_crawler_user_agent(user_agent: &str) -> bool {
    let ua = user_agent.to_ascii_lowercase();
    CRAWLER_MARKERS.iter().any(|m| ua.contains(m))
}

/// Whether the request being handled comes from a crawler. `false` outside a [`scope`]
/// (MCP calls, CLI tools, tests).
pub fn is_crawler() -> bool {
    CRAWLER.try_with(|crawler| *crawler).unwrap_or(false)
}

/// Run `fut` with the current request marked as coming from a crawler or not.
pub async fn scope<F: Future>(crawler: bool, fut: F) -> F::Output {
    CRAWLER.scope(crawler, fut).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_crawlers_are_detected() {
        let crawlers = [
            "Mozilla/5.0 (Linux; Android 6.0.1; Nexus 5X Build/MMB29P) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.7871.128 Mobile Safari/537.36 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
            "Mozilla/5.0 (Linux; Android 6.0.1; Nexus 5X Build/MMB29P) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.7871.186 Mobile Safari/537.36 (compatible; GoogleOther)",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15 (Applebot/0.1; +http://www.apple.com/go/applebot)",
            "Mozilla/5.0 (compatible; AhrefsBot/7.0; +http://ahrefs.com/robot/)",
            "Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm)",
            "AdsBot-Google (+http://www.google.com/adsbot.html)",
            "quillbot.com",
        ];
        for ua in crawlers {
            assert!(is_crawler_user_agent(ua), "{ua}");
        }
    }

    #[test]
    fn regular_clients_are_not_crawlers() {
        let others = [
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:150.0) Gecko/20100101 Firefox/150.0",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36",
            "curl/8.14.1",
            "node",
            "",
        ];
        for ua in others {
            assert!(!is_crawler_user_agent(ua), "{ua:?}");
        }
    }

    #[tokio::test]
    async fn is_crawler_follows_scope() {
        assert!(!is_crawler());
        assert!(scope(true, async { is_crawler() }).await);
        assert!(!scope(false, async { is_crawler() }).await);
        assert!(!is_crawler());
    }
}
