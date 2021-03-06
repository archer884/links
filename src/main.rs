use std::{
    fmt::Display,
    io::{self, Read},
};

use clap::Clap;
use hashbrown::HashSet;
use regex::Regex;

#[derive(Clap, Clone, Debug)]
struct Opts {
    base: Option<String>,
    #[clap(short, long)]
    filter: Option<String>,
}

#[derive(Clone, Debug)]
struct Extractor {
    href_pattern: Regex,
    url_pattern: Regex,
    www_pattern: Regex,
}

impl Extractor {
    fn new() -> Self {
        Self {
            href_pattern: Regex::new(r#"href="([^"]+)""#).unwrap(),
            url_pattern: Regex::new(r"(http|https)://[\w,./?'$%&*()+=-]+").unwrap(),
            www_pattern: Regex::new(r#"www\.[\w,./?'$%&*()+=-]+"#).unwrap(),
        }
    }

    fn extract<'a>(&'a self, content: &'a str) -> impl Iterator<Item = &str> {
        self.href_pattern
            .captures_iter(content)
            .map(|x| x.get(1).unwrap().as_str())
            .chain(self.url_pattern.find_iter(content).map(|x| x.as_str()))
            .chain(self.www_pattern.find_iter(content).map(|x| x.as_str()))
    }
}

impl Default for Extractor {
    fn default() -> Self {
        Self::new()
    }
}

/// A struct used to make saints.
#[derive(Clone, Debug, Default)]
struct Canonizer {
    base: Option<String>,
}

impl Canonizer {
    /// Make a string into a saint.
    fn canonize<'a, 'b>(&'b self, link: &'a str) -> Canonization<'a, 'b> {
        if link.starts_with("http") {
            return Canonization::Original(link);
        }

        if link.starts_with("www") {
            return Canonization::AddHttp(link);
        }

        self.base
            .as_ref()
            .map(|base| Canonization::AddBase(link, base))
            .unwrap_or_else(|| Canonization::Original(link))
    }
}

enum Canonization<'a, 'b> {
    AddBase(&'a str, &'b str),
    AddHttp(&'a str),
    Original(&'a str),
}

impl Display for Canonization<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Canonization::AddBase(link, base) => write!(f, "{}{}", base, link),
            Canonization::Original(link) => f.write_str(link),
            Canonization::AddHttp(link) => write!(f, "http://{}", link),
        }
    }
}

fn main() {
    let Opts { base, filter } = Opts::parse();
    let extractor = Extractor::new();
    let canonizer = Canonizer { base };
    let content = read_stdin();

    // Tragically, we need to realize these strings ahead of time in order to
    // filter duplicates.
    let links: Vec<_> = extractor
        .extract(&content)
        .map(|x| canonizer.canonize(x).to_string())
        .collect();

    let mut duplicate_filter = HashSet::new();
    let unique_links = links
        .iter()
        .filter(|&link| duplicate_filter.insert(protocol_agnostic(link.as_ref())));

    match filter {
        Some(filter) => display_filtered_links(&filter, unique_links),
        None => display_links(unique_links),
    }
}

fn display_links(links: impl IntoIterator<Item = impl AsRef<str>>) {
    for link in links {
        let link = link.as_ref();
        println!("{}", link);
    }
}

fn display_filtered_links(filter: &str, links: impl IntoIterator<Item = impl AsRef<str>>) {
    for link in links {
        let link = link.as_ref();
        if link.contains(filter) {
            println!("{}", link);
        }
    }
}

/// Strip the protocol from the left hand side of a url.
fn protocol_agnostic(link: &str) -> &str {
    link.find(':').map(|idx| &link[idx..]).unwrap_or(link)
}

fn read_stdin() -> String {
    let handle = io::stdin();
    let mut handle = handle.lock();
    let mut buf = String::new();
    handle.read_to_string(&mut buf).unwrap();
    buf
}

#[cfg(test)]
mod tests {
    use crate::Extractor;

    #[test]
    fn extractor_does_things() {
        let extractor = Extractor::new();
        let content = include_str!("../resources/input.html");
        let actual: Vec<_> = extractor.extract(content).collect();
        let expected = &[
            "/topics/rust",
            "/topics/infrastructure",
            "/topics/search",
            "/topics/index",
            "/topics/graph",
            "/topics/database",
            "/topics/server",
            "/topics/backend",
            "/topics/search-server",
            "http://foo.bar",
            "https://www.bar.baz",
            "http://foo.com/hello?name=Jack%20Ballenger&occupation=Barber",
            "http://foo.com/with/pluses+for+spaces",
            "http://foo.com/with/minuses-for-spaces",
            "www.bar.baz",
            "www.google.com",
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn wtf_is_going_on() {
        let a = b'-';
        let b = b'-';

        assert_eq!(a, b);
    }
}
