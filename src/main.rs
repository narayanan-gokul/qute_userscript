use ureq;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::fs::OpenOptions;
use std::process::{Command, Stdio};
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


#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct ItemSearchMatchLoginUri {
    r#match: Option<String>,
    uri: String,
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct ItemSearchLogin {
      username: String,
      password: String,
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct ItemSearchMatch {
    name: String,
    login: ItemSearchLogin,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ItemSearchData {
    object: String,
    data: Vec<ItemSearchMatch>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ItemSearchResponseBody {
    success: bool,
    data: ItemSearchData,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut only_username = false;
    let mut only_password = false;
    if args.len() == 2 {
        if args[1] == "-u" {
            only_username = true;
        } else if args[1] == "-p" {
            only_password = true;
        }
    }
    let status_check_response_body = ureq::get("http://localhost:8087/status")
        .call()?
        .body_mut()
        .read_json::<StatusResponseBody>()?;

    if status_check_response_body.data.template.status == "locked" {
        let password_input = Command::new("/home/gokul/.local/bin/dmenu")
            .args(&["-P", "-l", "5", "-p", "Enter master password"])
            .output()
            .expect("Failed to fetch master password");
        let password = String::from_utf8_lossy(&password_input.stdout)
            .trim()
            .to_string();

        let password_payload = PasswordPayload { password: password };
        ureq::post("http://localhost:8087/unlock")
            .header("Accept", "application/json")
            .send_json(&password_payload)?;
    }

    let url = env::var("QUTE_URL")
        .expect("No URL supplied");

    let item_search_response_body = ureq::get("http://localhost:8087/list/object/items")
        .header("Accept", "application/json")
        .query("url", url)
        .call()?
        .body_mut()
        .read_json::<ItemSearchResponseBody>()?;

    let results = item_search_response_body.data.data;

    let mut dmenu_string =  String::new();
    for result in &results {
        dmenu_string.push_str(&result.name);
        dmenu_string.push_str(" | ");
        dmenu_string.push_str(&result.login.username);
        dmenu_string.push('\n');
    }

    let mut selector_process = Command::new("/home/gokul/.local/bin/dmenu")
        .args(&["-l", "5", "-p", "Selecting item"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Unable to spawn dmenu process");

    let selector_stdin = selector_process.stdin
        .as_mut()
        .expect("Unable to open dmenu stdin");

    selector_stdin.write_all(&dmenu_string.into_bytes())
    .expect("Unable to write to stdin");

    let selector_output = selector_process.wait_with_output()
        .expect("Unable to collect output");

    let selection_string = String::from_utf8_lossy(&selector_output.stdout);
    let selection: Vec<&str> = selection_string.trim().split(" | ").collect();
    if selection.len() <= 1 {
        return Ok(());
    }
    let selected_item_name = selection[0];
    let selected_username = selection[1];

    let qute_fifo_path = env::var("QUTE_FIFO")
        .expect("No fifo path supplied");

    let mut qute_fifo = OpenOptions::new()
        .write(true)
        .open(&qute_fifo_path)
        .expect("Unable to open fifo file");

    for result in &results {
        if result.name == selected_item_name && result.login.username == selected_username {
            let mut fifo_string = format!("fake-key -g i")
                .to_string();
            if only_username {
                fifo_string.push_str(&selected_username);
                fifo_string.push_str("<Enter>");
            } else if only_password {
                fifo_string.push_str(&result.login.password);
                fifo_string.push_str("<Enter>");
            } else {
                fifo_string.push_str(&format!("{}<Tab>{}<Enter>",
                        selected_username,
                        result.login.password).to_string());
            }
            qute_fifo.write_all(&fifo_string.into_bytes())
                .expect("Unable to write to fifo");
            break;
        }
    }
    
    drop(qute_fifo);
    return Ok(());
}
