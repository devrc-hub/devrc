// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use crate::{
    diagnostics::Diagnostics,
    program_state::ProgramState,
    source_maps::{get_orig_position, CachedMaps},
};
use deno_core::{
    error::AnyError,
    serde_json,
    serde_json::{json, Value},
    OpState, ZeroCopyBuf,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

pub fn init(rt: &mut deno_core::JsRuntime) {
    super::reg_json_sync(rt, "op_apply_source_map", op_apply_source_map);
    super::reg_json_sync(rt, "op_format_diagnostic", op_format_diagnostic);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplySourceMap {
    file_name: String,
    line_number: i32,
    column_number: i32,
}

fn op_apply_source_map(
    state: &mut OpState,
    args: Value,
    _zero_copy: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
    let args: ApplySourceMap = serde_json::from_value(args)?;

    let mut mappings_map: CachedMaps = HashMap::new();
    let program_state = state.borrow::<Arc<ProgramState>>().clone();

    let (orig_file_name, orig_line_number, orig_column_number, _) = get_orig_position(
        args.file_name,
        args.line_number.into(),
        args.column_number.into(),
        &mut mappings_map,
        program_state,
    );

    Ok(json!({
      "fileName": orig_file_name,
      "lineNumber": orig_line_number as u32,
      "columnNumber": orig_column_number as u32,
    }))
}

fn op_format_diagnostic(
    _state: &mut OpState,
    args: Value,
    _zero_copy: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
    let diagnostic: Diagnostics = serde_json::from_value(args)?;
    Ok(json!(diagnostic.to_string()))
}
