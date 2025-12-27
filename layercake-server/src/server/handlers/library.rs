use axum::extract::{Multipart, Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::Value;

use layercake_core::database::entities::common_types::{
    DataType as CoreDataType, FileFormat as CoreFileFormat,
};
use crate::server::app::AppState;
use layercake_core::services::library_item_service::{LibraryItemService, ITEM_TYPE_PROJECT_TEMPLATE};

pub async fn download_library_item(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let service = LibraryItemService::new(state.db.clone());
    let item = service
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let filename = format!("{}-{}.bin", item.item_type, sanitize_filename(&item.name));
    let mut headers = HeaderMap::new();
    let content_type = item
        .content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    Ok((headers, item.content_blob))
}

pub async fn upload_library_item(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    use base64::Engine;

    let mut item_type: Option<String> = None;
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_format: Option<CoreFileFormat> = None;
    let mut data_type: Option<CoreDataType> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let key = field.name().unwrap_or("").to_string();
        match key.as_str() {
            "type" => {
                item_type = Some(field.text().await.unwrap_or_default());
            }
            "name" => {
                name = Some(field.text().await.unwrap_or_default());
            }
            "description" => {
                description = Some(field.text().await.unwrap_or_default());
            }
            "tags" => {
                if let Ok(raw) = field.text().await {
                    if let Ok(value) = serde_json::from_str::<Value>(&raw) {
                        if let Some(array) = value.as_array() {
                            tags = array
                                .iter()
                                .filter_map(|val| val.as_str().map(|s| s.to_string()))
                                .collect();
                        }
                    }
                }
            }
            "fileFormat" => {
                if let Ok(raw) = field.text().await {
                    file_format = raw.parse::<CoreFileFormat>().ok();
                }
            }
            "dataType" => {
                if let Ok(raw) = field.text().await {
                    data_type = raw.parse::<CoreDataType>().ok();
                }
            }
            "fileName" => {
                file_name = Some(field.text().await.unwrap_or_default());
            }
            "contentType" => {
                content_type = Some(field.text().await.unwrap_or_default());
            }
            "fileContent" => {
                if let Ok(raw) = field.text().await {
                    let bytes = base64::engine::general_purpose::STANDARD
                        .decode(raw.as_bytes())
                        .map_err(|_| StatusCode::BAD_REQUEST)?;
                    file_bytes = Some(bytes);
                }
            }
            "file" => {
                let file_name_value = field.file_name().map(|value| value.to_string());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?
                    .to_vec();
                file_bytes = Some(bytes);
                if file_name.is_none() {
                    file_name = file_name_value;
                }
            }
            _ => {}
        }
    }

    let item_type = item_type.ok_or(StatusCode::BAD_REQUEST)?;
    let name = name.ok_or(StatusCode::BAD_REQUEST)?;
    let file_name = file_name.ok_or(StatusCode::BAD_REQUEST)?;
    let file_bytes = file_bytes.ok_or(StatusCode::BAD_REQUEST)?;

    let service = LibraryItemService::new(state.db.clone());
    let actor = layercake_core::auth::SystemActor::internal();
    let tags_vec = tags;

    let result = match item_type.as_str() {
        "dataset" => {
            let file_format = file_format.ok_or(StatusCode::BAD_REQUEST)?;
            service
                .create_dataset_item(
                    &actor,
                    name,
                    description,
                    tags_vec,
                    file_name,
                    file_format,
                    data_type,
                    content_type,
                    file_bytes,
                )
                .await
        }
        ITEM_TYPE_PROJECT_TEMPLATE => {
            service
                .create_binary_item(
                    &actor,
                    ITEM_TYPE_PROJECT_TEMPLATE.to_string(),
                    name,
                    description,
                    tags_vec,
                    serde_json::json!({
                        "filename": file_name
                    }),
                    content_type,
                    file_bytes,
                )
                .await
        }
        _ => {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": result.id })),
    ))
}

fn sanitize_filename(input: &str) -> String {
    let filtered: String = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = filtered.trim_matches('_');
    if trimmed.is_empty() {
        "download".to_string()
    } else {
        trimmed.to_string()
    }
}
