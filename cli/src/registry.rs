use crate::{errors::DevrcResult, raw_devrcfile::RawDevrcfile};

#[derive(Debug, Clone, Default)]
pub struct Registry {
    pub files: Vec<RawDevrcfile>,
}

impl Registry {
    pub fn add(&mut self, file: RawDevrcfile) -> DevrcResult<()> {
        self.files.push(file);
        Ok(())
    }
}
