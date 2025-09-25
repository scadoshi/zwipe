use zwipe::inbound::http::handlers::auth::{HttpAuthenticateUser, HttpRegisterUser};

pub fn register_user(username: &str, email: &str, password: &str) {
    let request = HttpRegisterUser::new(username, email, password);
    todo!()
}

pub fn authenticate_user(identifier: &str, password: &str) {
    let request = HttpAuthenticateUser::new(identifier, password);
    todo!()
}
