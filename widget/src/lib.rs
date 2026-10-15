// From the aareguru widget from @eliabieri (https://github.com/eliabieri/wg_display_widget_aareguru/blob/main/widget/src/lib.rs)
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use serde_json::{json, Value};

use wg_display_widget::{export, http, Guest, WidgetContext, WidgetResult};

const WIDGET_NAME: &str = "Aareguru";

#[derive(JsonSchema, Deserialize)]
struct WidgetConfig {
    city: String,
}

struct MyWidget;

impl Guest for MyWidget {
    fn get_name() -> String {
        WIDGET_NAME.into()
    }

    fn run(context: WidgetContext) -> WidgetResult {
        if "{}" == context.config {
            return WidgetResult {
                data: "You must configure a city".into(),
            };
        }

        // Widgets can parse their config with ease using serde
        let Ok(config) = serde_json::from_str::<WidgetConfig>(&context.config) else {
            return WidgetResult {
                data: "Invalid config".into(),
            };
        };

        // Widgets can make network requests
        let response = http::request(
            http::Method::Get,
            format!(
                "https://aareguru.existenz.ch/v2018/today?city={}",
                config.city
            )
            .as_str(),
            None,
        );
        let Ok(response) = response else {
            return WidgetResult {
                data: "Failed to make network request".into(),
            };
        };

        if 200 != response.status {
            return WidgetResult {
                data: format!("Response status != 200: {}", response.status),
            };
        }

        let Ok(data) = serde_json::from_slice::<Value>(response.bytes.as_slice()) else {
            return WidgetResult {
                data: "Response from AareGuru could not be parsed".into(),
            };
        };

        WidgetResult {
            data: render(
                data["name"].as_str().unwrap_or(&config.city),
                data["aare"].as_f64().unwrap_or(0.0),
                data["text_short"].as_str().unwrap_or(""),
            ),
        }
    }

    fn get_config_schema() -> String  {
        let schema = schema_for!(WidgetConfig);
        serde_json::to_string_pretty(&schema).unwrap()
    }

    fn get_version() -> String {
        "2.0.0".into()
    }

    fn get_run_update_cycle_seconds() -> u32 {
        300
    }
}

fn temperature_color(temp: f64) -> u32 {
    match temp {
        t if t < 12.0 => 0xFF4FC3F7,
        t if t < 16.0 => 0xFF26C6DA,
        t if t < 19.0 => 0xFF66BB6A,
        t if t < 22.0 => 0xFFFFCA28,
        _ => 0xFFFF7043,
    }
}

/// import layout from main.slint by converting it with shared/scripts/slint_to_widget_elements.py
fn render(city: &str, temp: f64, text: &str) -> String {
    let accent = temperature_color(temp);
    json!({ "elements": [
        { "type": "rect", "x": 0.02, "y": 0.03, "w": 0.96, "h": 0.94,
          "background": 0, "border_color": accent, "border_width": 2,
          "border_radius": 14 },
        { "type": "text", "x": 0.06, "y": 0.07, "h": 0.15, "w": 0.88, "text": city,
          "font_size": 32, "font_weight": 700, "color": 0xFF9FB8CCu32,
          "horizontal_alignment": "center", "vertical_alignment": "center" },
        { "type": "text", "x": 0.06, "y": 0.22, "h": 0.42, "w": 0.88,
          "text": format!("{temp:.1} °C"), "font_size": 100, "font_weight": 700,
          "color": accent, "horizontal_alignment": "center",
          "vertical_alignment": "center" },
        { "type": "progress", "x": 0.08, "y": 0.68, "w": 0.84, "h": 0.05,
          "value": temp.clamp(0.0, 25.0), "max": 25.0, "fill": accent,
          "background": 0xFF1F3347u32 },
        { "type": "text", "x": 0.06, "y": 0.78, "h": 0.15, "w": 0.88, "text": text,
          "font_size": 30, "font_italic": true, "color": 0xFFE8F1F8u32,
          "horizontal_alignment": "center", "vertical_alignment": "center",
          "overflow": "elide" },
    ]})
    .to_string()
}

export!(MyWidget);
