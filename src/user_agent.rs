use crate::version;

pub fn get_user_agent() -> String {
    format!("devrc/{}", version::VERSION)
}
