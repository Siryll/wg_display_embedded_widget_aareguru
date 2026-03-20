// From the aareguru widget from @eliabieri (https://github.com/eliabieri/wg_display_widget_aareguru/blob/main/widget/src/lib.rs)
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use serde_json::{Error, Value};

use widget::widget::{http, logging};

wit_bindgen::generate!({
    path: "../wg_display_widget_wit/wit",
    world: "widget"
});

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
        let config: WidgetConfig =
            serde_json::from_str(&context.config).expect("Failed to parse config");

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

        let data: Result<Value, Error> = serde_json::from_slice(response.bytes.as_slice());
        let result = match data {
            Ok(data) => format!(
                "{}: {}°C ({})",
                config.city, data["aare"], data["text_short"]
            ),
            Err(_) => "Response from AareGuru could not be parsed".into(),
        };

        WidgetResult { data: result }
    }

    fn get_config_schema() -> String  {
        let schema = schema_for!(WidgetConfig);
        serde_json::to_string_pretty(&schema).unwrap()
    }

    fn get_version() -> String {
        "1.0.0".into()
    }

    fn get_run_update_cycle_seconds() -> u32 {
        300
    }
}

export!(MyWidget);
