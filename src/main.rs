#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(array_windows)]
#![deny(unsafe_op_in_unsafe_fn)]

use http::make_router;

mod generic_helpers;
mod http;
mod primitives;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;

    let router = make_router();

    axum::Server::bind(&"[::]:3000".parse()?)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
