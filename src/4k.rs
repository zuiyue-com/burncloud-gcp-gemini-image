use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use base64::Engine;
use dotenv::dotenv;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f64,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
    #[serde(rename = "responseModalities")]
    response_modalities: Vec<String>,
    #[serde(rename = "topP")]
    top_p: f64,
    #[serde(rename = "imageConfig")]
    image_config: ImageConfig,
}

#[derive(Serialize)]
struct ImageConfig {
    #[serde(rename = "aspectRatio")]
    aspect_ratio: String,
    #[serde(rename = "imageSize")]
    image_size: String,
    #[serde(rename = "imageOutputOptions")]
    image_output_options: ImageOutputOptions,
    #[serde(rename = "personGeneration")]
    person_generation: String,
}

#[derive(Serialize)]
struct ImageOutputOptions {
    #[serde(rename = "mimeType")]
    mime_type: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    #[serde(rename = "text")]
    text: Option<String>,
    #[serde(rename = "inlineData")]
    inline_data: Option<InlineData>,
}

#[derive(Deserialize)]
struct InlineData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 .env 文件
    dotenv().ok();

    let client = Client::new();

    // 从环境变量读取配置
    let api_url = env::var("GEMINI_API_URL")
        .unwrap_or_else(|_| "http://74.249.29.91:8080/v1beta/models/gemini-3-pro-image-preview:generateContent".to_string());
    let api_key = env::var("API_KEY")
        .expect("API_KEY must be set in .env file or environment variables");

    let request_body = GeminiRequest {
        contents: vec![
            Content {
                role: "user".to_string(),
                parts: vec![
                    Part {
                        text: "画条狗".to_string(),
                    }
                ],
            }
        ],
        generation_config: GenerationConfig {
            temperature: 1.0,
            max_output_tokens: 32768,
            response_modalities: vec!["TEXT".to_string(), "IMAGE".to_string()],
            top_p: 0.95,
            image_config: ImageConfig {
                aspect_ratio: "1:1".to_string(),
                image_size: "2K".to_string(),
                image_output_options: ImageOutputOptions {
                    mime_type: "image/png".to_string(),
                },
                person_generation: "ALLOW_ALL".to_string(),
            },
        },
    };

    println!("发送图像生成请求到 Gemini 3 Pro Image Preview...");

    let response = client
        .post(&format!("{}?key={}", api_url, api_key))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let result: GeminiResponse = response.json().await?;

        if let Some(candidate) = result.candidates.first() {
            for part in &candidate.content.parts {
                if let Some(inline_data) = &part.inline_data {
                    // 处理图片数据
                    let image_bytes = base64::engine::general_purpose::STANDARD.decode(&inline_data.data)?;

                    let file_extension = if inline_data.mime_type.contains("png") {
                        "png"
                    } else if inline_data.mime_type.contains("jpeg") || inline_data.mime_type.contains("jpg") {
                        "jpg"
                    } else if inline_data.mime_type.contains("gif") {
                        "gif"
                    } else if inline_data.mime_type.contains("webp") {
                        "webp"
                    } else {
                        "png"
                    };

                    let output_filename = format!("generated_dog.{}", file_extension);
                    std::fs::write(&output_filename, image_bytes)?;
                    println!("生成的狗图片已保存到 {} 文件", output_filename);
                }

                if let Some(text) = &part.text {
                    println!("AI回复文本: {}", text);
                    // 保存文本回复
                    std::fs::write("dog_generation_response.txt", text)?;
                    println!("文本回复已保存到 dog_generation_response.txt 文件");
                }
            }
        }

        println!("图像生成完成！");
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        println!("请求失败 - 状态码: {}", status);
        println!("错误内容: {}", error_text);

        // 保存错误信息
        std::fs::write("dog_generation_error.txt", format!("Status: {}\nError: {}", status, error_text))?;
        println!("错误信息已保存到 dog_generation_error.txt 文件");
    }

    Ok(())
}