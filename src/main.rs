use std::io::{self, Read};

use hashbrown::HashSet;
use linkex::LinkExtractor;
use structopt::StructOpt;

#[derive(Clone, Debug, structopt::StructOpt)]
struct Opts {
    source: Option<String>,
    #[structopt(short, long)]
    base: Option<String>,
    #[structopt(short, long)]
    filter: Option<String>,
}

fn main() {
    let Opts {
        source,
        base,
        filter,
    } = Opts::from_args();

    let content = match source {
        Some(url) => read_url(&url),
        None => read_stdin(),
    };

    let extractor = base
        .map(|base| LinkExtractor::with_base_url(base))
        .unwrap_or_default();

    // Evaluate canonization to check for duplicates.
    let links: Vec<_> = extractor.extract(&content).map(|x| x.to_string()).collect();
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

fn read_url(url: &str) -> String {
    ureq::get(url).call().unwrap().into_string().unwrap()
}
