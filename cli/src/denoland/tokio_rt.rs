// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use std::future;

use tokio::runtime;

use crate::errors::DevrcResult;

pub fn create_runtime() -> runtime::Runtime {
    runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        // This limits the number of threads for blocking operations (like for
        // synchronous fs ops) or CPU bound tasks like when we run dprint in
        // parallel for deno fmt.
        // The default value is 512, which is an unhelpfully large thread pool. We
        // don't ever want to have more than a couple dozen threads.
        .max_blocking_threads(32)
        .build()
        .unwrap()
}

pub fn run<F, R>(future: F) -> DevrcResult<R>
where
    F: future::Future<Output = R>,
{
    let rt = create_runtime();
    Ok(rt.block_on(future))
}
