use std::str::FromStr;

use anyhow::{Context, Result};
use rusoto_core::{Client, HttpClient, Region};
use rusoto_core::credential::{ChainProvider, ProfileProvider};
use rusoto_route53::{Change, ChangeBatch, ChangeResourceRecordSetsRequest, GetChangeRequest, ResourceRecord, ResourceRecordSet, Route53, Route53Client};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
enum RecordType {
    A,
    CNAME,
    TXT,
}

impl FromStr for RecordType {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(RecordType::A),
            "CNAME" => Ok(RecordType::CNAME),
            "TXT" => Ok(RecordType::TXT),
            _ => Err(std::fmt::Error::default()),
        }
    }
}

impl ToString for RecordType {
    fn to_string(&self) -> String {
        match self {
            RecordType::A => "A",
            RecordType::CNAME => "CNAME",
            RecordType::TXT => "TXT",
        }
        .into()
    }
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "update-record")]
    UpdateRecord {
        #[structopt(name = "dns-name")]
        dns_name: String,

        /// Record Type
        #[structopt(name = "record-type")]
        record_type: RecordType,

        /// Value(s)
        #[structopt(name = "value", long, required = true)]
        value: Vec<String>,

        /// TTL (in seconds)
        #[structopt(name = "ttl", long)]
        ttl: i64,

        /// Comment
        #[structopt(name = "comment", long)]
        comment: Option<String>,
    },
}

#[derive(StructOpt, Debug)]
struct CmdLine {
    /// Zone ID
    #[structopt(name = "hosted-zone-id")]
    hosted_zone_id: String,

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

    match args.command {
        Command::UpdateRecord {
            dns_name,
            record_type,
            value,
            ttl,
            comment,
        } => update_record(
            &client,
            &args.hosted_zone_id,
            &dns_name,
            &record_type,
            &value,
            ttl,
            &comment,
        ),
    }
}

fn update_record(
    client: &Route53Client,
    zone: &str,
    dns_name: &str,
    record_type: &RecordType,
    values: &[String],
    ttl: i64,
    comment: &Option<String>,
) -> Result<()> {
    let result = client.change_resource_record_sets(ChangeResourceRecordSetsRequest {
        change_batch: ChangeBatch {
            changes: vec![Change {
                action: "UPSERT".to_string(),
                resource_record_set: ResourceRecordSet {
                    alias_target: None,
                    failover: None,
                    geo_location: None,
                    health_check_id: None,
                    multi_value_answer: None,
                    name: dns_name.to_string(),
                    region: None,
                    resource_records: Some(
                        values
                            .iter()
                            .map(|v| ResourceRecord {
                                value: v.to_string(),
                            })
                            .collect(),
                    ),
                    set_identifier: None,
                    ttl: Some(ttl),
                    traffic_policy_instance_id: None,
                    type_: record_type.to_string(),
                    weight: None,
                },
            }],
            comment: comment.clone(),
        },
        hosted_zone_id: zone.to_string(),
    }).sync().context("unable to execute ChangeResourceRecordSets")?;

    let mut id = result.change_info.id;
    if id.contains("/") {
        let parts = id.splitn(2, "/").collect::<Vec<&str>>();
        id = parts[1].to_string();
    }
    let mut current_status = result.change_info.status;
    while current_status != "INSYNC" {
        std::thread::sleep(std::time::Duration::from_secs(5));

        let result = client.get_change(GetChangeRequest {
            id: (&id).clone()
        }).sync().with_context(|| format!("unable to GetChangeRequest({})", id))?;
        current_status = result.change_info.status
    }

    Ok(())
}
