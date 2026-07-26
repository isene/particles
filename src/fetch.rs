//! One-time article fetch. The particle table itself is compiled in —
//! seventeen fundamental particles do not change — so the only thing
//! worth caching is the prose.

use crate::data::{Cache, PARTICLES};
use std::io::Write;
use std::sync::{Arc, Mutex};

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(60))
        .user_agent("particles/0.1 (https://github.com/isene/particles)")
        .build()
}

pub fn fetch_all() -> Result<Cache, String> {
    let total = PARTICLES.len();
    println!("Fetching the Wikipedia article for all {total} particles …");
    let cache = Arc::new(Mutex::new(Cache::default()));
    let next = Arc::new(Mutex::new(0usize));
    let done = Arc::new(Mutex::new(0usize));
    let mut workers = Vec::new();
    for _ in 0..3 {
        let cache = Arc::clone(&cache);
        let next = Arc::clone(&next);
        let done = Arc::clone(&done);
        workers.push(std::thread::spawn(move || {
            let agent = agent();
            loop {
                let i = {
                    let mut n = next.lock().unwrap();
                    let i = *n;
                    *n += 1;
                    i
                };
                if i >= total {
                    break;
                }
                let p = &PARTICLES[i];
                if let Ok((title, text)) = fetch_article(&agent, p.wiki) {
                    let mut c = cache.lock().unwrap();
                    c.articles.insert(p.name.to_string(), text);
                    c.sources.insert(
                        p.name.to_string(),
                        format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_")),
                    );
                }
                let mut d = done.lock().unwrap();
                *d += 1;
                print!("\r  [{:2}/{}] {:<20}", *d, total, p.name);
                std::io::stdout().flush().ok();
                std::thread::sleep(std::time::Duration::from_millis(60));
            }
        }));
    }
    for w in workers {
        let _ = w.join();
    }
    let cache = Arc::try_unwrap(cache)
        .map_err(|_| "worker thread leaked")?
        .into_inner()
        .unwrap();
    println!("\r  {} articles fetched{:20}", cache.articles.len(), "");
    Ok(cache)
}

fn fetch_article(agent: &ureq::Agent, title: &str) -> Result<(String, String), String> {
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&prop=extracts&explaintext=1&redirects=1&format=json&formatversion=2&titles={}",
        urlencode(title)
    );
    let mut last = String::new();
    for attempt in 0..4 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(400 << attempt));
        }
        match agent.get(&url).call() {
            Ok(resp) => match resp.into_json::<serde_json::Value>() {
                Ok(json) => {
                    let page = &json["query"]["pages"][0];
                    let t = page["title"].as_str().unwrap_or(title).to_string();
                    let text = page["extract"].as_str().unwrap_or("").trim().to_string();
                    if text.is_empty() {
                        return Err("empty extract".into());
                    }
                    return Ok((t, text));
                }
                Err(e) => last = e.to_string(),
            },
            Err(ureq::Error::Status(code, _)) if code == 429 || code >= 500 => {
                last = format!("http {code}");
            }
            Err(e) => {
                last = e.to_string();
                break;
            }
        }
    }
    Err(last)
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
