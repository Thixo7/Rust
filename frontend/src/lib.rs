use yew::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Alert {
    ip: String,
    message: String,
    timestamp: String,
}

#[function_component(App)]
fn app() -> Html {
    let alerts = use_state(|| vec![]);

    {
        let alerts = alerts.clone();
        use_effect_with((), move |_| {
            let alerts = alerts.clone();
            spawn_local(async move {
                let result = Request::get("http://127.0.0.1:8080/alerts")
                    .send()
                    .await;

                if let Ok(resp) = result {
                    if let Ok(parsed) = resp.json::<Vec<Alert>>().await {
                        alerts.set(parsed);
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div>
            <h1>{"Rust IDS - Alertes détectées"}</h1>
            <table border="1">
                <thead>
                    <tr>
                        <th>{"IP"}</th>
                        <th>{"Message"}</th>
                        <th>{"Horodatage"}</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        for (*alerts).iter().map(|alert: &Alert| html! {
                            <tr>
                                <td>{ &alert.ip }</td>
                                <td>{ &alert.message }</td>
                                <td>{ &alert.timestamp }</td>
                            </tr>
                        })
                    }
                </tbody>
            </table>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    console_log::init_with_level(log::Level::Debug).expect("log init failed");
    yew::Renderer::<App>::new().render();
}
