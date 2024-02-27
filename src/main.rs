use warp::Filter;
use extism::*;
use std::collections::HashMap;
use std::fs;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct InitialState {
    message: String,
    count: i32,
}

#[tokio::main]
async fn main() {

    let elements = read_elements("./elements"); 
    let initial_state = InitialState {
        message: "Hello from initialState".to_string(),
        count: 42,
    };

    let data = json!({
        "markup": "<my-header>Hello World</my-header>",
        "initialState": initial_state,
        "elements": elements,
    });

    let enhanced_json = enhance(&data);
    let document_html = enhanced_json["document"].as_str().unwrap_or_default().to_string();

    // Use Arc to share the HTML content across requests.
    let shared_document_html = Arc::new(document_html);

    // Define the route
    let hello_world = warp::path!("hello" / "world")
        .map({
            let shared_document_html = shared_document_html.clone();
            move || {
                let response_html = shared_document_html.clone();
                warp::reply::html(response_html.to_string())
            }
        });

    // Start the server
    println!("Starting server on http://localhost:3030");
    println!("Enhanced page at http://localhost:3030/hello/world");
    warp::serve(hello_world)
        .run(([127, 0, 0, 1], 3030))
        .await;
}


fn read_elements(directory: &str) -> HashMap<String, String> {
    let mut elements = HashMap::new();
    let paths = fs::read_dir(directory).unwrap_or_else(|err| {
        panic!("Error reading directory: {}", err);
    });

    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!("Error reading file {:?}: {}", path, err);
            });
            let file_stem = path.file_stem().unwrap().to_str().unwrap().to_owned();
            elements.insert(file_stem, content);
        }
    }

    elements
}


fn enhance(data: &serde_json::Value) -> serde_json::Value {
  let enhance_wasm = Wasm::file( "./wasm/enhance-ssr.wasm");
  let manifest = Manifest::new([enhance_wasm]);
  let mut plugin = Plugin::new(&manifest, [], true).unwrap();


  let input = serde_json::to_string(data).expect("Failed to serialize data");
  let res = plugin.call::<&str, &str>("ssr", &input).unwrap();
  serde_json::from_str(&res).expect("Failed to deserialize plugin response")
}

