# pixiv3-rs

[![Crates.io](https://img.shields.io/crates/v/pixiv3-rs)](https://crates.io/crates/pixiv3-rs)
[![docs.rs](https://img.shields.io/docsrs/pixiv3-rs)](https://docs.rs/pixiv3-rs)
[![License: MIT](https://img.shields.io/crates/l/pixiv3-rs)](LICENSE)
[![CI](https://github.com/GeminiLab/pixiv3-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/GeminiLab/pixiv3-rs/actions/workflows/ci.yml)

Rust client for the [Pixiv App API](https://www.pixiv.net/) (app-api.pixiv.net). Port of the Python library [pixivpy3](https://github.com/upbit/pixivpy), with a similar public API, and an updated asynchronous structure.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
pixiv3-rs = "0.1"
futures-util = "0.3"
```

Example: create a client with a refresh token and fetch user details and illustrations:

```rust
use pixiv3_rs::{AppPixivAPI, PixivError};
use futures_util::{StreamExt, pin_mut};

#[tokio::main]
async fn main() -> Result<(), PixivError> {
    let api = AppPixivAPI::new_from_refresh_token("YOUR_REFRESH_TOKEN".into());

    let user_detail = api.user_detail(11, None, true).await?;
    println!("{:?}", user_detail);

    let illust_iter = api.user_illusts_iter(11, None, None, None, true);

    pin_mut!(illust_iter);
    while let Some(illust) = illust_iter.next().await? {
        println!("{:?}", illust);
    }

    Ok(())
}
```

Without authentication (e.g. for public endpoints or when using a proxy that injects auth):

```rust
let api = AppPixivAPI::new_no_auth();
```

With a fixed access token:

```rust
let api = AppPixivAPI::new_from_access_token("your_access_token".into());
```

## Features

- **`stream`** (default): Enables streaming helpers and async iteration where applicable.
- **`log`** (default): Enables logging via the `log` crate. Disable with `default-features = false` for a dependency-free build if you do not need logging.

## Relation to pixivpy3

This library is a port of [upbit/pixivpy](https://github.com/upbit/pixivpy), with a similar public API, and an updated architecture:

- **Method names**: Kept close to pixivpy3 (e.g. `user_detail`, `user_illusts`, `illust_detail`) for easier migration.
- **Async-first**: All API calls are async and return `Result<T, PixivError>`; use `tokio` (or another runtime) to run them.
- **Streaming**: Optional `stream` feature provides async iterators (e.g. `user_illusts_iter`) instead of manual pagination.
- **Macro-based definitions**: API endpoints (most of them now) are defined using a procedural macro, to avoid boilerplate code.

## License

[Unlicense](./LICENSE), same as [upbit/pixivpy](https://github.com/upbit/pixivpy). And I quote its words here:

> Feel free to use, reuse and abuse the code in this project.
