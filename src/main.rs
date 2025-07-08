use ureq;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::env;

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
struct StatusResponseDataTemplate {
    serverUrl: Option<String>,
    lastSync: String,
    userEmail: String,
    userId: String,
    status: String,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
struct StatusResponseData {
    object: String,
    template: StatusResponseDataTemplate,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
struct StatusResponseBody {
    success: bool,
    data: StatusResponseData,
}

#[derive(Serialize)]
struct PasswordPayload {
    password: String,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
struct UnlockResponseData {
    noColor: bool,
    object: String,
    title: String,
    message: String,
    raw: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct UnlockResponseBody {
    success: bool,
    data: UnlockResponseData,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let status_check_response_body = ureq::get("http://localhost:8087/status")
        .call()?
        .body_mut()
        .read_json::<StatusResponseBody>()?;

    if status_check_response_body.data.template.status == "locked" {
        let password_input = Command::new("dmenu")
            .args(&["-P", "-l", "5", "-p", "Enter master password"])
            .output()
            .expect("Failed to fetch master password");
        let password = String::from_utf8_lossy(&password_input.stdout).trim().to_string();

        let password_payload = PasswordPayload { password: password };
        let unlock_response_body = ureq::post("http://localhost:8087/unlock")
            .header("Accept", "application/json")
            .send_json(&password_payload)?
            .body_mut()
            .read_json::<UnlockResponseBody>()?;

        println!("{}", unlock_response_body.data.title)
    }

    return Ok(());
}
