// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

pub mod errors;
pub mod runtime_compiler;

use deno_core::{
    error::AnyError, json_op_async, json_op_sync, serde_json::Value, BufVec, JsRuntime, OpState,
    ZeroCopyBuf,
};
use deno_runtime::metrics::metrics_op;
use std::{cell::RefCell, future::Future, rc::Rc};

pub fn reg_json_async<F, R>(rt: &mut JsRuntime, name: &'static str, op_fn: F)
where
    F: Fn(Rc<RefCell<OpState>>, Value, BufVec) -> R + 'static,
    R: Future<Output = Result<Value, AnyError>> + 'static,
{
    rt.register_op(name, metrics_op(name, json_op_async(op_fn)));
}

pub fn reg_json_sync<F>(rt: &mut JsRuntime, name: &'static str, op_fn: F)
where
    F: Fn(&mut OpState, Value, &mut [ZeroCopyBuf]) -> Result<Value, AnyError> + 'static,
{
    rt.register_op(name, metrics_op(name, json_op_sync(op_fn)));
}
