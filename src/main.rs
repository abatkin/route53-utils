use anyhow::Result;
use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_route53::config::Region;
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
#[structopt(
    name = "route53-util",
    about = "Utilities for working with AWS Route53",
    author = "Adam Batkin"
)]
struct CmdLine {
    /// AWS Profile
    #[structopt(name = "profile", long)]
    profile: Option<String>,

    /// AWS Region
    #[structopt(name = "region", long)]
    region: Option<String>,

    /// Command
    #[structopt(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CmdLine::from_args();

    let mut credentials_provider_builder = DefaultCredentialsChain::builder();
    if let Some(profile_name) = args.profile {
        credentials_provider_builder =
            credentials_provider_builder.profile_name(profile_name.as_ref());
    }
    let credentials_provider = credentials_provider_builder.build().await;

    let region_provider =
        RegionProviderChain::first_try(args.region.map(Region::new)).or_default_provider();

    let config = aws_config::from_env()
        .credentials_provider(credentials_provider)
        .region(region_provider)
        .load()
        .await;

    let client = aws_sdk_route53::Client::new(&config);

    let result = match args.command {
        Command::UpdateRecord(params) => update_record(&client, params).await,
        Command::WaitForChange(params) => wait_for_change::wait_for_change(&client, params).await,
    }?;
    let code = if result { 0 } else { 1 };
    ::std::process::exit(code)
}
