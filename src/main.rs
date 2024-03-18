use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use serde_json::Value;
use serde::{de::DeserializeOwned, Deserialize, Serialize};


// todo use serde to only have one struct
#[derive(Debug, Serialize, Deserialize)]
struct MenuItemFields {
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MenuItem {
    fields: MenuItemFields,
}

// todo use serde to only have one struct
#[derive(Debug, Serialize, Deserialize)]
struct TagsItemFields {
    tag: String,
    sort_order: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct TagsItem {
    fields: TagsItemFields,
}

#[derive(Debug, Serialize, Deserialize)]
struct AirtableResponse <T> {
    records: Vec<T>,
    offset: Option<String>,
}

async fn get_table<T : DeserializeOwned > (airtable_api_key: &str, table_id: &str) -> Result<Vec<T>, Error> {
    let menu_url = format!("https://api.airtable.com/v0/appFsXK6zp1O2SyV8/{}", table_id);
    let client = reqwest::Client::new();
    let mut ret = vec![];
    let mut offset = "0".to_string();
    loop {
        let airtable_resp = client.get(&menu_url)
            .query(&[("offset", &offset)])
            .header("Authorization", format!("Bearer {}", airtable_api_key))
            .send().await?;
        if airtable_resp.status() != 200 {
            panic!("Airtable API returned status code {} {}", airtable_resp.status(), airtable_resp.text().await?);
        }
        let airtable_response: AirtableResponse<T> = airtable_resp.json().await?;
        ret.extend(airtable_response.records.into_iter());
        match airtable_response.offset {
            Some(o) => offset = o,
            None => break,
        }
    }
        
    Ok(ret)
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<Value>) -> Result<(), Error> {
    // Extract some useful information from the request
    // read AIRTABLE_API_KEY
    let airtable_api_key = std::env::var("AIRTABLE_API_KEY").expect("AIRTABLE_API_KEY must be set");
    
    // todo this could be done in parallel
    let menu = get_table::<MenuItem>(&airtable_api_key, "tblHWvgCofYT2Puac").await?;
    let tags = get_table::<TagsItem>(&airtable_api_key, "tblC2T0WC2MTR3MQ3").await?;
    
    // todo # create new tags (POST)

    dbg!(event);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
