//! The few runtime services the SDK needs, on native targets (tokio) and in
//! the browser (WebAssembly timers).

use std::time::Duration;

pub async fn sleep(duration: Duration) {
    #[cfg(feature = "native")]
    tokio::time::sleep(duration).await;
    #[cfg(all(not(feature = "native"), target_arch = "wasm32"))]
    gloo_timers::future::sleep(duration).await;
    #[cfg(all(not(feature = "native"), not(target_arch = "wasm32")))]
    std::thread::sleep(duration);
}
