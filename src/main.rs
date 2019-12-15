use anyhow::{Context, Result};
use rusoto_core::credential::{ChainProvider, ProfileProvider};
use rusoto_core::{Client, HttpClient, Region};
use rusoto_route53::Route53Client;
use structopt::StructOpt;

use update_record::*;
use wait_for_change::*;

mod update_record;
mod wait_for_change;

#[derive(StructOpt, Debug)]
enum Command {
    /// Update a Resource Record within a zone
    #[structopt(name = "update-record")]
    UpdateRecord(UpdateRecordParams),

    /// Wait for a change (by ID)
    #[structopt(name = "wait-for-change")]
    WaitForChange(WaitForChangeParams),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "route53-util", about = "Utilities for working with AWS Route53", author = "Adam Batkin")]
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
        Command::WaitForChange(params) => wait_for_change::wait_for_change(&client, params),
    }?;
    let code = if result { 0 } else { 1 };
    ::std::process::exit(code)
}
