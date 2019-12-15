use anyhow::{Context, Result};
use rusoto_route53::{Change, ChangeBatch, ChangeResourceRecordSetsRequest, GetChangeRequest, ResourceRecord, ResourceRecordSet, Route53, Route53Client};
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
}


pub fn update_record(client: &Route53Client, params: UpdateRecordParams) -> Result<()> {
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
                    name: params.dns_name.to_string(),
                    region: None,
                    resource_records: Some(
                        params.value
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
    }).sync().context("unable to execute ChangeResourceRecordSets")?;

    let mut id = result.change_info.id;
    if id.contains('/') {
        let parts = id.splitn(2, '/').collect::<Vec<&str>>();
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