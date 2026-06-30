use serde_json::{json, Value};
use uuid::Uuid;
use wreq::Client as HttpClient;

use crate::auth::Session;
use crate::endpoints;
use crate::error::Error;

const MODEL_ID: &str = "56fdd199312815e2";

pub async fn generate(
    http: &HttpClient,
    session: &Session,
    authuser: u32,
    prompt: &str,
) -> Result<String, Error> {
    let turn_uuid = Uuid::new_v4().to_string().to_uppercase();
    let model_uuid = Uuid::new_v4().to_string().to_uppercase();
    let req_hex = Uuid::new_v4().simple().to_string();

    let inner = build_inner(prompt, &turn_uuid, &req_hex);
    let inner_str = serde_json::to_string(&inner)?;
    let freq = json!([Value::Null, inner_str]);
    let freq_str = serde_json::to_string(&freq)?;

    let url = endpoints::generate(authuser);
    let model_header = format!(
        "[1,null,null,null,\"{MODEL_ID}\",null,null,0,[4,5,6,8],null,null,2,null,null,1,1,\"{model_uuid}\"]"
    );
    let turn_header = format!("[\"{turn_uuid}\",1]");

    let resp = http
        .post(url.as_str())
        .query(&[
            ("bl", session.build_label.as_str()),
            ("f.sid", session.session_id.as_str()),
            ("hl", "en"),
            ("pageId", "none"),
            ("_reqid", "100000"),
            ("rt", "c"),
        ])
        .header("Origin", "https://gemini.google.com")
        .header("Referer", "https://gemini.google.com/")
        .header("X-Same-Domain", "1")
        .header("x-goog-ext-525001261-jspb", model_header.as_str())
        .header("x-goog-ext-525005358-jspb", turn_header.as_str())
        .header("x-goog-ext-73010989-jspb", "[0]")
        .header("x-goog-ext-73010990-jspb", "[0,0,0]")
        .form(&[
            ("at", session.access_token.as_str()),
            ("f.req", freq_str.as_str()),
        ])
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        return Err(Error::BadStatus {
            status: status.as_u16(),
        });
    }

    let body = resp.text().await?;
    Ok(body)
}

fn build_inner(prompt: &str, turn_uuid: &str, req_hex: &str) -> Value {
    let message_content = json!([prompt, 0, null, null, null, null, 0]);
    let metadata = json!(["", "", "", null, null, null, null, null, null, ""]);

    let mut inner = vec![Value::Null; 92];
    inner[0] = message_content;
    inner[1] = json!(["en"]);
    inner[2] = metadata;
    inner[4] = json!(req_hex);
    inner[6] = json!([1]);
    inner[7] = json!(1);
    inner[10] = json!(1);
    inner[11] = json!(0);
    inner[17] = json!([[0]]);
    inner[18] = json!(0);
    inner[27] = json!(1);
    inner[30] = json!([4]);
    inner[41] = json!([1]);
    inner[53] = json!(0);
    inner[59] = json!(turn_uuid);
    inner[61] = json!([]);
    inner[67] = json!(0);
    inner[68] = json!(2);
    inner[79] = json!(1);
    inner[80] = json!(1);
    inner[91] = json!(0);

    Value::Array(inner)
}
