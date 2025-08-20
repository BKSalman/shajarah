use serde::{Deserialize, Serialize};
#[derive(Debug, garde::Validate, Serialize, Deserialize, Clone)]
pub struct RegisterInput {
    #[garde(skip)]
    pub first_name: String,
    #[garde(skip)]
    pub last_name: String,
    #[garde(email)]
    pub email: String,
    #[garde(skip)]
    pub password: String,
}
#[derive(Debug, garde::Validate, Serialize, Deserialize, Clone)]
pub struct LoginInput {
    #[garde(email)]
    pub email: String,
    #[garde(skip)]
    pub password: String,
}
