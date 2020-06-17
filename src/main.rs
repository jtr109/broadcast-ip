#[macro_use]
extern crate clap;
use chrono::{Local, SecondsFormat};
use clap::App;
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

fn ifconfig(interface: &str) -> String {
    let output = Command::new("ifconfig").arg(interface).output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Millis, false)
}

async fn new_issue(
    issue_api: &str,
    title: &str,
    description: &str,
    private_token: &str,
) -> Response {
    let encoded_title: String = utf8_percent_encode(&title, FRAGMENT).collect();
    let encoded_description: String = utf8_percent_encode(&description, FRAGMENT).collect();
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

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let issue_api = matches.value_of("api").unwrap();
    let private_token = matches.value_of("token").unwrap();

    let title = format!("[{}] Network Config of Raspberry Pi", now());
    let description = format!("```\n{}\n```", ifconfig("wlan0"));

    let response = new_issue(issue_api, &title, &description, private_token).await;
    let status = response.status();
    let content = response.text().await.unwrap();
    log::info!(
        "[{}] {}",
        status.as_str(),
        web_url_from_response_content(&content),
    );
}
