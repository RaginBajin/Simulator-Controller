use arrow::array::{Array, BooleanArray, Float64Array, Int32Array, Int64Array};
use arrow::record_batch::RecordBatch;
use serde::Serialize;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataResponse {
    pub session_id: String,
    pub sample_count: usize,
    pub channels: Vec<ChannelData>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelData {
    pub name: String,
    pub values: Vec<serde_json::Value>,
}

#[tauri::command]
pub async fn get_chart_data(
    session_id: String,
    start_distance: Option<f64>,
    end_distance: Option<f64>,
    state: State<'_, AppState>,
) -> Result<ChartDataResponse, AppError> {
    let batch = match (start_distance, end_distance) {
        (Some(start), Some(end)) => {
            state
                .db
                .read_telemetry_window(&session_id, start, end)
                .await
                .map_err(AppError::from)?
        }
        _ => {
            state
                .db
                .read_telemetry(&session_id)
                .await
                .map_err(AppError::from)?
        }
    };

    let response = batch_to_chart_response(&session_id, &batch);
    Ok(response)
}

fn batch_to_chart_response(session_id: &str, batch: &RecordBatch) -> ChartDataResponse {
    let schema = batch.schema();
    let mut channels = Vec::with_capacity(schema.fields().len());

    for (i, field) in schema.fields().iter().enumerate() {
        let col = batch.column(i);
        let values = column_to_json_values(col);
        channels.push(ChannelData {
            name: field.name().clone(),
            values,
        });
    }

    ChartDataResponse {
        session_id: session_id.to_string(),
        sample_count: batch.num_rows(),
        channels,
    }
}

fn column_to_json_values(col: &dyn Array) -> Vec<serde_json::Value> {
    use arrow::datatypes::DataType;

    let null_fill = || (0..col.len()).map(|_| serde_json::Value::Null).collect();

    match col.data_type() {
        DataType::Float64 => {
            let Some(arr) = col.as_any().downcast_ref::<Float64Array>() else {
                return null_fill();
            };
            arr.iter()
                .map(|v| match v {
                    Some(val) => serde_json::Value::from(val),
                    None => serde_json::Value::Null,
                })
                .collect()
        }
        DataType::Int64 => {
            let Some(arr) = col.as_any().downcast_ref::<Int64Array>() else {
                return null_fill();
            };
            arr.iter()
                .map(|v| match v {
                    Some(val) => serde_json::Value::from(val),
                    None => serde_json::Value::Null,
                })
                .collect()
        }
        DataType::Int32 => {
            let Some(arr) = col.as_any().downcast_ref::<Int32Array>() else {
                return null_fill();
            };
            arr.iter()
                .map(|v| match v {
                    Some(val) => serde_json::Value::from(val),
                    None => serde_json::Value::Null,
                })
                .collect()
        }
        DataType::Boolean => {
            let Some(arr) = col.as_any().downcast_ref::<BooleanArray>() else {
                return null_fill();
            };
            arr.iter()
                .map(|v| match v {
                    Some(val) => serde_json::Value::from(val),
                    None => serde_json::Value::Null,
                })
                .collect()
        }
        _ => null_fill(),
    }
}
