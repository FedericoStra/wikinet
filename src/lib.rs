use html5ever::rcdom::{Handle, NodeData};
use soup::prelude::*;

#[tracing::instrument(skip(html))]
pub fn get_first_link(html: &str) -> Option<Handle> {
    let soup = Soup::new(html);
    let mw_content_text = soup
        .tag("div")
        .attr("id", "mw-content-text")
        .find()
        .expect("cannot find #mw-content-text");
    let mw_parser_output = mw_content_text
        .tag("div")
        .class("mw-parser-output")
        .find()
        .expect("cannot find .mw-parser-output");
    for p in mw_parser_output.tag("p").recursive(false).find_all() {
        {
            let p = p.display();
            if p.len() <= 64 {
                tracing::debug!("p = {:?}", p);
            } else {
                tracing::debug!("p = {:?} ...", &p[0..64]);
            }
        }
        let mut parens: i32 = 0;
        for c in p.children.borrow().iter() {
            match c.data {
                NodeData::Text { ref contents } => {
                    let text: &str = &contents.borrow()[..];
                    let opening = text.matches('(').count() as i32;
                    let closing = text.matches(')').count() as i32;
                    parens += opening - closing;
                    tracing::trace!(opening, closing, "Text: parentheses:");
                }
                NodeData::Element {
                    ref name,
                    ref attrs,
                    ..
                } => {
                    if parens > 0 {
                        tracing::trace!(parens, "Link: skipping:");
                        continue;
                    }
                    let tag: &str = &name.local[..];
                    if tag == "a" {
                        for attr in attrs.borrow().iter() {
                            let html5ever::tree_builder::Attribute { name, value } = attr;
                            if &name.local[..] == "href" {
                                if let Some(_) = normalize_href(&value[..]) {
                                    let link = &value[..];
                                    tracing::debug!(link, "found");
                                    return Some(c.clone());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}

#[tracing::instrument]
pub async fn get_wiki(path: &str) -> reqwest::Result<String> {
    let mut url = "https://en.wikipedia.org".to_string();
    url.push_str(path);
    tracing::debug!(?url, "getting");
    let response = reqwest::get(&url).await?;
    let text = response.text().await?;
    Ok(text)
}

pub fn normalize_href(href: &str) -> Option<&str> {
    if href.starts_with("/wiki/") {
        let href = href.trim_start_matches("/wiki/");
        if href.starts_with("Help:IPA/") {
            None
        } else {
            Some(href)
        }
    } else {
        None
    }
}

pub fn trim_wiki_website(href: &str) -> &str {
    href.trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_start_matches("en.wikipedia.org")
    // .trim_start_matches("/wiki/")
}

pub fn ensure_wiki_at_start(href: &str) -> String {
    if href.starts_with("/wiki/") {
        href.to_string()
    } else {
        let mut new = "/wiki/".to_string();
        new.push_str(href);
        new
    }
}
