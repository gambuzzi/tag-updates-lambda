use std::collections::HashSet;

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
struct AirtablePayload <T> {
    records: Vec<T>,
    #[serde(skip_serializing_if="Option::is_none")]    
    offset: Option<String>,
}

async fn get_table<T : DeserializeOwned > (airtable_api_key: &str, table_id: &str) -> Result<Vec<T>, Error> {
    let menu_url = format!("https://api.airtable.com/v0/app2iGoHc7c47uF5J/{}", table_id);
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
        let airtable_response: AirtablePayload<T> = airtable_resp.json().await?;
        ret.extend(airtable_response.records.into_iter());
        match airtable_response.offset {
            Some(o) => offset = o,
            None => break,
        }
    }
        
    Ok(ret)
}

async fn post_table<T : Serialize> (airtable_api_key: &str, table_id: &str, payload: &AirtablePayload<T>) -> Result<(), Error> {
    let base_url = format!("https://api.airtable.com/v0/app2iGoHc7c47uF5J/{}", table_id);
    let client = reqwest::Client::new();

    let resp = client.post(&base_url)
        .header("Authorization", format!("Bearer {}", airtable_api_key))
        .json(payload)
        .send().await?;
    if resp.status() == 200 {
        Ok(())
    } else {
        dbg!(resp.text().await?);   
        //todo fix error types
        panic!("Airtable creation failed")
    }
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
    let menu = get_table::<MenuItem>(&airtable_api_key, "tbl4MeUd1X997rBkN").await?;
    let tags = get_table::<TagsItem>(&airtable_api_key, "tblZSCExfKX9WFT0E").await?;
    
    // create new tags (POST)
    let mut max_sort_order = tags.iter().map(|t| t.fields.sort_order).max().unwrap_or(0);

    // todo filter before flatmap
    let menu_tags_set: HashSet<String> = menu.iter().flat_map(|m| m.fields.tags.clone().unwrap_or_default()).collect();
    let tags_set: HashSet<String> = tags.iter().map(|m| m.fields.tag.clone()).collect();

    let tags_to_create: Vec<&String> = menu_tags_set.difference(&tags_set).collect();
    // {"records": [{"fields": {"tag": tag, "sort_order": max_sort_order}}]}

    for chunk in tags_to_create.chunks(10) {
        let payload = AirtablePayload {
            records: chunk.iter().enumerate().map(|(idx, t)| 
                TagsItem {
                fields: TagsItemFields {
                    tag: t.to_string(),
                    sort_order: max_sort_order + 10*(idx+1) as u32,
                }
            }).collect(),
            offset: None
        };
        post_table(&airtable_api_key, "tblZSCExfKX9WFT0E", &payload).await?;
        max_sort_order += 10*(chunk.len() as u32);
    }
    dbg!(event);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
