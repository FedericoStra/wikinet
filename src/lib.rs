use html5ever::rcdom::{Handle, Node, NodeData};
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

        let span = tracing::debug_span!("asd").entered();
        for handle in iter_links(html) {
            let data = &handle.data;
            match data {
                NodeData::Element { name, attrs, .. } => {
                    tracing::debug!(?name.local, ?attrs);
                }
                _ => {}
            }
        }
        span.exit();

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
                            let key: &str = &name.local[..];
                            if key == "href" {
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

pub fn filter_valid_links<'a>(
    nodes: impl Iterator<Item = &'a Node>,
) -> impl Iterator<Item = &'a Node> {
    nodes
        .scan(0, |&mut parens, c| {
            if let NodeData::Text { ref contents } = c.data {
                let text: &str = &contents.borrow()[..];
                let opening = text.matches('(').count() as i32;
                let closing = text.matches(')').count() as i32;
                tracing::trace!(opening, closing, "Text: parentheses:");
                Some((parens + opening - closing, c))
            } else {
                Some((parens, c))
            }
        })
        .filter_map(|(parens, c)| {
            if let NodeData::Element { name, attrs, .. } = &c.data {
                if parens > 0 {
                    return None;
                }
                let tag: &str = &name.local[..];
                if tag == "a" {
                    for attr in attrs.borrow().iter() {
                        let html5ever::tree_builder::Attribute { name, value } = attr;
                        let key: &str = &name.local[..];
                        if key == "href" {
                            if let Some(_) = normalize_href(&value[..]) {
                                let link = &value[..];
                                tracing::debug!(link, "found");
                                return Some(c.clone());
                            }
                        }
                    }
                }
            }
            return None;
        })
}

#[tracing::instrument(skip(html))]
pub fn iter_links(html: &str) -> impl Iterator<Item = Handle> {
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
    let paragraphs = mw_parser_output.tag("p").recursive(false).find_all();

    paragraphs.flat_map(|p: Handle| {
        p.children
            .borrow()
            .iter()
            .scan(0, |&mut parens, c| {
                if let NodeData::Text { ref contents } = c.data {
                    let text: &str = &contents.borrow()[..];
                    let opening = text.matches('(').count() as i32;
                    let closing = text.matches(')').count() as i32;
                    tracing::trace!(opening, closing, "Text: parentheses:");
                    Some((parens + opening - closing, c))
                } else {
                    Some((parens, c))
                }
            })
            .filter_map(|(parens, c)| {
                if let NodeData::Element { name, attrs, .. } = &c.data {
                    if parens > 0 {
                        return None;
                    }
                    let tag: &str = &name.local[..];
                    if tag == "a" {
                        for attr in attrs.borrow().iter() {
                            let html5ever::tree_builder::Attribute { name, value } = attr;
                            let key: &str = &name.local[..];
                            if key == "href" {
                                if let Some(_) = normalize_href(&value[..]) {
                                    let link = &value[..];
                                    tracing::debug!(link, "found");
                                    return Some(c.clone());
                                }
                            }
                        }
                    }
                }
                return None;
            })
            .collect::<Vec<_>>()
    })
}

#[tracing::instrument]
pub async fn get_wiki(search: &str) -> eyre::Result<String> {
    let mut path = std::path::PathBuf::from("/wiki/");
    path.push(search);

    let mut url = reqwest::Url::parse("https://en.wikipedia.org")?;
    url.set_path(&path.to_string_lossy()[..]);

    {
        let url = url.as_str();
        tracing::debug!(url, "getting");
    }

    let response = reqwest::get(url).await?;
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
