use std::future::Future;

use crate::domain::user::models::{
    CreateUser, CreateUserError, DeleteUser, DeleteUserError, GetUser, GetUserError, UpdateUser,
    UpdateUserError, User,
};

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        request: &CreateUser,
    ) -> impl Future<Output = Result<User, CreateUserError>> + Send;

    fn get_user(
        &self,
        request: &GetUser,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    fn update_user(
        &self,
        request: &UpdateUser,
    ) -> impl Future<Output = Result<User, UpdateUserError>> + Send;

    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}

pub trait UserService: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        request: &CreateUser,
    ) -> impl Future<Output = Result<User, CreateUserError>> + Send;

    fn get_user(
        &self,
        request: &GetUser,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    fn update_user(
        &self,
        request: &UpdateUser,
    ) -> impl Future<Output = Result<User, UpdateUserError>> + Send;

    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}
