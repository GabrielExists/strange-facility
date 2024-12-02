mod app;
mod jobs;

#[cfg(target_arch = "wasm32")]
use app::App;

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!(r#"Please don't run this manually, instead use "cargo tauri dev""#)

}