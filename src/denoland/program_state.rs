// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use deno_core::ModuleSpecifier;
use deno_lib::file_fetcher::File;

/// Simple struct implementing in-process caching to prevent multiple
/// fs reads/net fetches for same file.
#[derive(Clone, Default, Debug)]
pub struct FileCache(Arc<Mutex<HashMap<ModuleSpecifier, File>>>);

impl FileCache {
    pub fn get(&self, specifier: &ModuleSpecifier) -> Option<File> {
        let cache = self.0.lock().unwrap();
        cache.get(specifier).cloned()
    }

    pub fn insert(&self, specifier: ModuleSpecifier, file: File) -> Option<File> {
        let mut cache = self.0.lock().unwrap();
        cache.insert(specifier, file)
    }
}

#[derive(Debug, Default)]
pub struct FileFetcher {
    pub cache: FileCache,
}

impl FileFetcher {
    pub fn insert_cached(&self, file: File) -> Option<File> {
        self.cache.insert(file.specifier.clone(), file)
    }
}

#[derive(Debug, Default)]
pub struct DenoDevrcProgramState {
    pub file_fetcher: FileFetcher,
    // pub maybe_import_map: Option<ImportMap>,
    // pub dino_dir: deno_dir::DenoDir
}
