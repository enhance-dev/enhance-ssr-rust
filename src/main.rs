use warp::Filter;
use extism::*;
use std::collections::HashMap;
use std::fs;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::sync::Arc;
use std::path::Path;

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
        "markup": "<my-header>Hello World</my-header><nest-my-header>nested</nest-my-header><another-header>another</another-header><more-header>more</more-header><most-header>most</most-header>",
        "initialState": initial_state,
        "elements": elements,
    });

    let enhanced_json = enhance(&data);
    let document_html = enhanced_json["document"].as_str().unwrap_or_default().to_string();
    let body_html = enhanced_json["body"].as_str().unwrap_or_default().to_string();
    let style_css = enhanced_json["styles"].as_str().unwrap_or_default().to_string();

    let body_owned: String = body_html.to_owned();
    let style_owned: String = style_css.to_owned();

    let together = format!("<html><head><link href='/static/index.css' rel='stylesheet'></link><style>{style_owned}</style></head><body>{body_owned}<script type='module' src='/static/index.js'></script></body></html>");
    println!("{}", together);

    // Use Arc to share the HTML content across requests.
    let shared_document_html = Arc::new(document_html);
    let constructed_document_html = Arc::new(together);

    // Define the route
    let hello_world = warp::path!("hello" / "world")
        .map({
            let shared_document_html = shared_document_html.clone();
            move || {
                let response_html = shared_document_html.clone();
                warp::reply::html(response_html.to_string())
            }
        });

    let hello_constructed = warp::path!("hello" / "constructed")
        .map({
            let constructed_document_html = constructed_document_html.clone();
            move || {
                let response_html = constructed_document_html.clone();
                warp::reply::html(response_html.to_string())
            }
        });


    // Start the server
    println!("Starting server on http://localhost:3030");
    println!("Enhanced page at http://localhost:3030/hello/world");
    println!("Constructed Enhanced page at http://localhost:3030/hello/constructed");
    let static_route = warp::path("static")
        .and(warp::fs::dir("./www/static/"));
    let routes = readme.or(static_route);
    let routes = warp::get().and(
        hello_world
            .or(hello_constructed)
            .or(static_route),
    );
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}


fn enhance(data: &serde_json::Value) -> serde_json::Value {
  let enhance_wasm = Wasm::file( "./wasm/enhance-ssr.wasm");
  let manifest = Manifest::new([enhance_wasm]);
  let mut plugin = Plugin::new(&manifest, [], true).unwrap();


  let input = serde_json::to_string(data).expect("Failed to serialize data");
  let res = plugin.call::<&str, &str>("ssr", &input).unwrap();
  serde_json::from_str(&res).expect("Failed to deserialize plugin response")
}


fn read_elements(directory: &str) -> HashMap<String, String> {
    let mut elements = HashMap::new();
    let base_path = Path::new(directory);
    read_directory(base_path, base_path, &mut elements);
    elements
}

fn read_directory(base_path: &Path, current_path: &Path, elements: &mut HashMap<String, String>) {
    if let Ok(entries) = fs::read_dir(current_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                read_directory(base_path, &path, elements);
            } else {
                match path.extension().and_then(|s| s.to_str()) {
                    Some("mjs") | Some("js") | Some("html") => {
                        let content = fs::read_to_string(&path).unwrap_or_else(|err| {
                            panic!("Error reading file {:?}: {}", path, err);
                        });
                        let key = generate_key(base_path, &path);
                        let processed_content = match path.extension().and_then(|s| s.to_str()) {
                            Some("html") => format!(r#"function ({{html, state}}){{return html`{}`}}"#, content),
                            _ => content,
                        };
                        elements.insert(key, processed_content);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn generate_key(base_path: &Path, path: &Path) -> String {
    let relative_path = path.strip_prefix(base_path).unwrap();
    let maybe_parent = relative_path.parent();
    let file_stem = path.file_stem().unwrap().to_str().unwrap();

    match maybe_parent {
        Some(parent) if parent != Path::new("") => {
            let parent_str = parent.to_str().unwrap().replace("/", "-").replace("\\", "-");
            format!("{}-{}", parent_str, file_stem)
        },
        _ => {
            file_stem.to_owned()
        }
    }
}
