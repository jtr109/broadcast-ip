use chrono::{Local, SecondsFormat};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwest::Response;
use serde_json::Value;
use std::env;
use std::process::Command;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'%')
    .add(b'+');
const PRIVATE_TOKEN: &str = "PRIVATE-TOKEN";

fn ifconfig() -> String {
    let output = Command::new("ifconfig").output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Millis, false)
}

async fn new_issue(
    issue_api: String,
    title: String,
    description: String,
    private_token: String,
) -> Response {
    let encoded_title: String = utf8_percent_encode(&title, FRAGMENT).collect();
    let encoded_description: String = utf8_percent_encode(&description, FRAGMENT).collect();
    // let encoded_description = "hello".to_string();
    let mut url = reqwest::Url::parse(&issue_api).unwrap();
    url.set_query(Some(&format!(
        "title={}&description={}",
        encoded_title, encoded_description,
    )));
    let client = reqwest::Client::new();
    client
        .post(url)
        .header(PRIVATE_TOKEN, private_token)
        .send()
        .await
        .unwrap()
}

fn web_url_from_response_content(content: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(content) {
        return v["web_url"].to_string();
    }
    content.to_string()
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let issue_api = format!(
        "{}/api/v4/projects/{}/issues",
        env::var("BROADCAST_IP_GITLAB").unwrap(),
        env::var("BROADCAST_IP_PROJECT_ID").unwrap(),
    );
    let private_token = env::var("BROADCAST_IP_TOKEN").unwrap();
    let title = format!("[{}] Network Config of Raspberry Pi", now());
    let description = format!("```\n{}\n```", ifconfig());

    let response = new_issue(issue_api, title, description, private_token).await;
    let status = response.status();
    let content = response.text().await.unwrap();
    log::info!(
        "[{}] {}",
        status.as_str(),
        web_url_from_response_content(&content),
    );
}
