use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _stdout_subscriber = tracing_subscriber::fmt::init();
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    // let embed_client = configuration::helpers::load_any_embed_client(&config);
    // let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    // let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    let Some(config) = config.data_access.embed.openai.as_ref() else {
        return Err("No OpenAI config found".into());
    };

    // =====================================================

    let client = Client::new();

    // =============== Responses
    let body = json!({
      "model": "gpt-4.1",
      "input": "What is the weather like in Boston today?",
      "tools": [
        {
          "type": "function",
          "name": "get_current_weather",
          "description": "Get the current weather in a given location",
          "parameters": {
            "type": "object",
            "properties": {
              "location": {
                "type": "string",
                "description": "The city and state, e.g. San Francisco, CA"
              },
              "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"]
              }
            },
            "required": ["location", "unit"]
          }
        }
      ],
      "tool_choice": "auto"
    });

    let response = client
        .post("https://api.openai.com/v1/responses")
        .header("Content-Type", "application/json")
        .bearer_auth(config.api_key.clone())
        .json(&body)
        .send()
        .await?
        .text()
        .await?;

    // =============== Chat Completion
    // let body = json!({
    //     "model": "gpt-5-nano-2025-08-07",
    //     "messages": [
    //         { "role": "user", "content": "what was a positive news story from today?" }
    //     ]
    // });

    // let response = client
    //     .post("https://api.openai.com/v1/chat/completions")
    //     .header("Content-Type", "application/json")
    //     .bearer_auth(config.api_key.clone())
    //     .json(&body)
    //     .send()
    //     .await?
    //     .text()
    //     .await?;

    println!("{}", response);

    Ok(())
}
