use anyhow::{Context, Result};
use aws_sdk_route53::types::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType,
};
use structopt::StructOpt;

use crate::wait_for_change::{is_change_complete, wait_for_completion};

#[derive(StructOpt, Debug)]
pub struct UpdateRecordParams {
    /// Zone ID
    #[structopt(name = "zone", long)]
    pub hosted_zone_id: String,

    /// Fully-qualified DNS Name
    #[structopt(name = "name", long)]
    pub dns_name: String,

    /// Record Type (i.e. A, CNAME, TXT, etc...)
    #[structopt(name = "type", long)]
    pub record_type: String,

    /// Action (CREATE, DELETE, UPSERT)
    #[structopt(name = "action", long, default_value = "UPSERT")]
    pub action: String,

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

pub async fn update_record(
    client: &aws_sdk_route53::Client,
    params: UpdateRecordParams,
) -> Result<bool> {
    let result = client
        .change_resource_record_sets()
        .change_batch(
            ChangeBatch::builder()
                .changes(
                    Change::builder()
                        .action(ChangeAction::from(params.action.as_str()))
                        .resource_record_set(
                            ResourceRecordSet::builder()
                                .name(params.dns_name)
                                .set_resource_records(Some(
                                    params
                                        .value
                                        .iter()
                                        .map(|v| ResourceRecord::builder().value(v).build())
                                        .collect(),
                                ))
                                .ttl(params.ttl)
                                .r#type(RrType::from(params.record_type.as_str()))
                                .build(),
                        )
                        .build(),
                )
                .set_comment(params.comment)
                .build(),
        )
        .hosted_zone_id(params.hosted_zone_id)
        .send()
        .await
        .context("unable to update zone")?;

    let change_id = result.change_info().unwrap().id().unwrap();
    println!("Change sent, Id {}", change_id);

    if params.no_wait {
        Ok(true)
    } else if is_change_complete(result.change_info.as_ref().unwrap()) {
        println!("Complete!");
        Ok(true)
    } else {
        let mut id = change_id.to_string();
        if id.contains('/') {
            let parts = id.splitn(2, '/').collect::<Vec<&str>>();
            id = parts[1].to_string();
        }

        wait_for_completion(client, &id, params.sleep_time, params.max_wait).await
    }
}
