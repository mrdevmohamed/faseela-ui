//! Native and browser entry points share exactly the same application initialization.
use crate::app::Showcase;
use faseela_ui::theme::{self, ThemePreset};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn native() -> eframe::Result<()> {
    eframe::run_native(
        "Faseela",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            theme::apply_preset(&cc.egui_ctx, ThemePreset::FaseelaDark);
            theme::install_arabic_font(&cc.egui_ctx);
            Ok(Box::<Showcase>::default())
        }),
    )
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("browser document is unavailable"))?;
        let canvas = document
            .get_element_by_id("faseela-ui-canvas")
            .ok_or_else(|| JsValue::from_str("showcase canvas was not found"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;

        wasm_bindgen_futures::spawn_local(async move {
            let result = eframe::WebRunner::new()
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|cc| {
                        theme::apply_preset(&cc.egui_ctx, ThemePreset::FaseelaDark);
                        theme::install_arabic_font(&cc.egui_ctx);
                        let app = Box::<Showcase>::default();
                        // The app creator runs during WebRunner initialization, after the
                        // renderer is ready. Report readiness after the app has been created
                        // rather than coupling it to the runner future's completion.
                        report_ready();
                        Ok(app)
                    }),
                )
                .await;

            if let Err(error) = result {
                web_sys::console::error_1(&JsValue::from_str(&format!(
                    "Faseela failed to start: {error:?}"
                )));
            }
        });
        Ok(())
    }

    fn report_ready() {
        match web_sys::Event::new("FaseelaUiReady") {
            Ok(event) => {
                if let Some(window) = web_sys::window() {
                    if let Err(error) = window.dispatch_event(&event) {
                        web_sys::console::error_1(&JsValue::from_str(&format!(
                            "Faseela initialized but could not report readiness: {error:?}"
                        )));
                    }
                }
            }
            Err(error) => web_sys::console::error_1(&JsValue::from_str(&format!(
                "Faseela initialized but could not create its readiness event: {error:?}"
            ))),
        }
    }
}
