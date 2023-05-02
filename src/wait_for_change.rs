use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use aws_sdk_route53::types::ChangeInfo;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct WaitForChangeParams {
    /// Change ID
    #[structopt(name = "change-id", long)]
    pub change_id: String,

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

pub async fn wait_for_change(
    client: &aws_sdk_route53::Client,
    params: WaitForChangeParams,
) -> Result<bool> {
    wait_for_completion(
        client,
        &params.change_id,
        params.sleep_time,
        if params.no_wait { 0 } else { params.max_wait },
    )
    .await
}

pub async fn wait_for_completion(
    client: &aws_sdk_route53::Client,
    change_id: &str,
    sleep_time: u64,
    max_wait: u64,
) -> Result<bool> {
    let start_time = Instant::now();
    loop {
        if check_for_completion(client, change_id).await? {
            println!("Complete!");
            return Ok(true);
        }

        println!("Not complete yet");

        let now = Instant::now();
        let duration = now - start_time;
        if duration > Duration::from_secs(max_wait) {
            println!(
                "Timed out waiting for completion of change Id {}",
                change_id
            );
            break;
        }

        std::thread::sleep(std::time::Duration::from_secs(sleep_time));
    }
    Ok(false)
}

pub async fn check_for_completion(
    client: &aws_sdk_route53::Client,
    change_id: &str,
) -> Result<bool> {
    let result = client
        .get_change()
        .id(change_id)
        .send()
        .await
        .with_context(|| format!("failed to check completion for change_id {}", change_id))?;
    Ok(is_change_complete(&result.change_info.unwrap()))
}

pub fn is_change_complete(change_info: &ChangeInfo) -> bool {
    change_info.status().unwrap().as_str() == "INSYNC"
}
