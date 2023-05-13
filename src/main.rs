use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use clap::{Parser};
use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::LevelFilter;

use trust_dns_resolver::proto::rr::RecordType;
use trust_dns_resolver::Resolver;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short = 'd', value_name = "Domain",
    conflicts_with("domainList"),
    required_unless_present("domainList"),
    value_delimiter(','),
    help("domain, accept multiple values"),
    next_display_order = 1
    )]
    domain: Option<Vec<String>>,

    #[clap(short = 'l', value_name = "Domain List",
    conflicts_with("domain"),
    required_unless_present("domain"),
    help("domain list file"),
    next_display_order = 2
    )]
    domain_list: Option<PathBuf>,

    #[clap(short = 't', value_name = "A|CNAME|MX", default_value = "A", next_display_order = 3)]
    lookup_type: Option<String>,
}

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(LevelFilter::ERROR))
        .init();
    tracing::debug!("Command args: {:?}",std::env::args_os());
    let cli: Cli = Cli::parse();
    tracing::debug!("Parsed command args: {:#?}",cli);

    let domainlist: Vec<String>;
    let record_type: RecordType;

    match cli.domain_list {
        Some(domain_list_file) => {
            let fp = File::open(domain_list_file).unwrap();
            let freader = BufReader::new(fp);

            domainlist = freader
                .lines()
                .into_iter()
                .map(|i| i.unwrap_or("".to_string()).trim().to_string())
                .filter(|i| i.len() > 0)
                .take(100_000)   // 限制域名数量
                // .inspect(|i| tracing::debug!(" -- {}",i.to_string()))
                .collect();
        },
        None => {
            domainlist = cli.domain.unwrap_or(vec![]);
        },
    }

    record_type = match cli.lookup_type.unwrap_or("".to_string()).as_ref() {
        "A" => RecordType::A,
        "CNAME" => RecordType::CNAME,
        "MX" => RecordType::MX,
        _ => RecordType::A,
    };

    tracing::debug!("All domains: {:?}", domainlist);

    let sys_resolver = Resolver::from_system_conf().expect("Failed while get dns server from system.");

    for domain in domainlist{
        let resp = sys_resolver.lookup(domain.clone(), record_type).expect(&format!("Filed while lookup {}", domain));
        let _ = resp.iter()
            .for_each(|ans| println!("{}", ans.to_string()));
            // .for_each(|ans| println!("{}\t{}",domain, ans.to_string()));
    }



}