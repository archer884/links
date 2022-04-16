use std::io::{self, Read};

use clap::Parser;
use hashbrown::HashSet;
use extractor::LinkExtractor;

#[derive(Clone, Debug, Parser)]
struct Opts {
    base: Option<String>,
    
    /// source url
    #[clap(short, long)]
    source: Option<String>,
}

fn main() {
    let opts = Opts::parse();
    if let Err(e) = run(&opts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(opts: &Opts) -> anyhow::Result<()> {
    let extractor = opts.base.as_ref().map(LinkExtractor::with_base_url).unwrap_or_default();
    let text = opts.source.as_ref()
        .map(|url| reqwest::blocking::get(url).and_then(|cx| cx.text()))
        .transpose()?
        .unwrap_or_else(read_stdin);

    // Evaluate canonization to check for duplicates.
    let links: Vec<_> = extractor.extract(&text).map(|x| x.to_string()).collect();
    let mut duplicate_filter = HashSet::new();
    let unique_links = links
        .iter()
        .filter(|&link| duplicate_filter.insert(strip_protocol(link.as_ref())));

    display_links(unique_links);
    Ok(())
}

fn display_links(links: impl IntoIterator<Item = impl AsRef<str>>) {
    for link in links {
        let link = link.as_ref();
        println!("{}", link);
    }
}

fn strip_protocol(link: &str) -> &str {
    link.find(':').map(|idx| &link[idx..]).unwrap_or(link)
}

fn read_stdin() -> String {
    let handle = io::stdin();
    let mut handle = handle.lock();
    let mut buf = String::new();
    handle.read_to_string(&mut buf).unwrap();
    buf
}
