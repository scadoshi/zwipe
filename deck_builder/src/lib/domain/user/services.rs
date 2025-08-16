use crate::domain::user::models::{User, UserCreationError, UserCreationRequest};

pub trait UserService {
    async fn create_user(&self, req: &UserCreationRequest) -> Result<User, UserCreationError>;
}
