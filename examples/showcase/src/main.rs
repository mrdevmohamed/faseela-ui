mod app;
mod startup;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    startup::native()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
