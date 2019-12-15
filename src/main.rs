use anyhow::{Context, Result};
use rusoto_core::credential::{ChainProvider, ProfileProvider};
use rusoto_core::{Client, HttpClient, Region};
use rusoto_route53::Route53Client;
use structopt::StructOpt;

use update_record::*;

mod update_record;

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "update-record")]
    UpdateRecord(UpdateRecordParams),
}

#[derive(StructOpt, Debug)]
struct CmdLine {
    /// AWS Profile
    #[structopt(name = "profile", long)]
    profile: Option<String>,

    /// AWS Region
    #[structopt(name = "region", long)]
    region: Region,

    /// Command
    #[structopt(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let args = CmdLine::from_args();

    let mut profile_provider =
        ProfileProvider::new().context("unable to construct profile provider")?;
    args.profile
        .iter()
        .for_each(|profile| profile_provider.set_profile(profile));
    let credential_provider = ChainProvider::with_profile_provider(profile_provider);
    let http_client = HttpClient::new().context("unable to construct http client")?;
    let client = Route53Client::new_with_client(
        Client::new_with(credential_provider, http_client),
        args.region,
    );

    let result = match args.command {
        Command::UpdateRecord(params) => update_record(&client, params),
    }?;
    let code = if result { 0 } else { 1 };
    ::std::process::exit(code)
}
