use anyhow::{Context, Result};
use rusoto_route53::{
    Change, ChangeBatch, ChangeInfo, ChangeResourceRecordSetsRequest, GetChangeRequest,
    ResourceRecord, ResourceRecordSet, Route53, Route53Client,
};
use std::time::{Duration, Instant};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct UpdateRecordParams {
    /// Zone ID
    #[structopt(name = "zone", long)]
    pub hosted_zone_id: String,

    /// Fully-qualified DNS Name
    #[structopt(name = "name", long)]
    pub dns_name: String,

    /// Record Type
    #[structopt(name = "type", long)]
    pub record_type: String,

    /// Value(s)
    #[structopt(name = "value", long, required = true)]
    pub value: Vec<String>,

    /// Time-to-live (in seconds)
    #[structopt(name = "ttl", long)]
    pub ttl: i64,

    /// Comment
    #[structopt(name = "comment", long)]
    pub comment: Option<String>,

    /// Do not wait for completion
    #[structopt(name = "no-wait", long)]
    pub no_wait: bool,

    /// Sleep time (in seconds) to sleep between completion checks
    #[structopt(name = "sleep", long, default_value = "5")]
    pub sleep_time: u64,

    /// Maximum time (in seconds) to wait for completion, before giving up
    #[structopt(name = "max-wait", long, default_value = "120")]
    pub max_wait: u64,
}

pub fn update_record(client: &Route53Client, params: UpdateRecordParams) -> Result<bool> {
    let result = client
        .change_resource_record_sets(ChangeResourceRecordSetsRequest {
            change_batch: ChangeBatch {
                changes: vec![Change {
                    action: "UPSERT".to_string(),
                    resource_record_set: ResourceRecordSet {
                        alias_target: None,
                        failover: None,
                        geo_location: None,
                        health_check_id: None,
                        multi_value_answer: None,
                        name: params.dns_name.to_string(),
                        region: None,
                        resource_records: Some(
                            params
                                .value
                                .iter()
                                .map(|v| ResourceRecord {
                                    value: v.to_string(),
                                })
                                .collect(),
                        ),
                        set_identifier: None,
                        ttl: Some(params.ttl),
                        traffic_policy_instance_id: None,
                        type_: params.record_type.to_string(),
                        weight: None,
                    },
                }],
                comment: params.comment.clone(),
            },
            hosted_zone_id: params.hosted_zone_id.to_string(),
        })
        .sync()
        .context("unable to execute ChangeResourceRecordSets")?;
    println!("Change sent, Id {}", result.change_info.id);

    if params.no_wait {
        Ok(true)
    } else if is_complete(&result.change_info) {
        println!("Complete!");
        Ok(true)
    } else {
        let mut id = result.change_info.id;
        if id.contains('/') {
            let parts = id.splitn(2, '/').collect::<Vec<&str>>();
            id = parts[1].to_string();
        }

        wait_for_completion(client, &id, params.sleep_time, params.max_wait)
    }
}

pub fn wait_for_completion(
    client: &Route53Client,
    change_id: &str,
    sleep_time: u64,
    max_wait: u64,
) -> Result<bool> {
    let start_time = Instant::now();
    loop {
        if check_for_completion(client, change_id)? {
            println!("Complete!");
            return Ok(true);
        }

        println!("Not complete yet");

        std::thread::sleep(std::time::Duration::from_secs(sleep_time));

        let now = Instant::now();
        let duration = now - start_time;
        if duration > Duration::from_secs(max_wait) {
            println!("Timed out waiting for completion of change Id {}", change_id);
            break;
        }
    }
    Ok(false)
}

fn check_for_completion(client: &Route53Client, change_id: &str) -> Result<bool> {
    let result = client
        .get_change(GetChangeRequest {
            id: change_id.to_string(),
        })
        .sync()
        .with_context(|| format!("unable to GetChangeRequest({})", change_id))?;
    Ok(is_complete(&result.change_info))
}

fn is_complete(change_info: &ChangeInfo) -> bool {
    change_info.status == "INSYNC"
}
