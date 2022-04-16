use std::fmt::Display;

use regex::Regex;

pub struct LinkExtractor {
    base_url: Option<String>,
    href_pattern: Regex,
}

impl LinkExtractor {
    pub fn new() -> Self {
        Self {
            base_url: None,
            href_pattern: Regex::new(r#"href="([^"]+)""#).unwrap(),
        }
    }

    pub fn with_base_url(url: impl Into<String>) -> Self {
        let mut url = url.into();
        if url.ends_with('/') {
            url.truncate(url.len() - 1);
        }

        Self {
            base_url: Some(url),
            ..Default::default()
        }
    }

    pub fn extract<'a>(&'a self, content: &'a str) -> impl Iterator<Item = Canonization<'a>> {
        self.href_pattern
            .captures_iter(content)
            .filter_map(move |captures| {
                let url = captures.get(1)?.as_str();
                Some(self.canonize(url))
            })
    }

    fn canonize<'a>(&'a self, url: &'a str) -> Canonization<'a> {
        if url.starts_with("www") {
            return Canonization::AddHttp(url);
        }

        if self.base_url.is_none() || url.starts_with("http") {
            return Canonization::Original(url);
        }

        self.base_url
            .as_ref()
            .map(|base| Canonization::AddBase(url, base))
            .unwrap_or_else(|| Canonization::Original(url))
    }
}

impl Default for LinkExtractor {
    fn default() -> Self {
        LinkExtractor::new()
    }
}

pub enum Canonization<'a> {
    AddBase(&'a str, &'a str),
    AddHttp(&'a str),
    Original(&'a str),
}

impl Display for Canonization<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Canonization::AddBase(link, base) => {
                let link = link.trim_start_matches('/');
                write!(f, "{}/{}", base, link)
            }
            Canonization::Original(link) => f.write_str(link),
            Canonization::AddHttp(link) => write!(f, "http://{}", link),
        }
    }
}

#[cfg(test)]
mod tests {}
