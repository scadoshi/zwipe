use std::future::Future;

use crate::domain::user::models::{
    CreateUserError, CreateUserRequest, DeleteUserError, DeleteUserRequest, GetUserError,
    GetUserRequest, UpdateUserError, UpdateUserRequest, User,
};

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        request: &CreateUserRequest,
    ) -> impl Future<Output = Result<User, CreateUserError>> + Send;

    fn get_user(
        &self,
        request: &GetUserRequest,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    fn update_user(
        &self,
        request: &UpdateUserRequest,
    ) -> impl Future<Output = Result<User, UpdateUserError>> + Send;

    fn delete_user(
        &self,
        request: &DeleteUserRequest,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}

pub trait UserService: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        request: &CreateUserRequest,
    ) -> impl Future<Output = Result<User, CreateUserError>> + Send;

    fn get_user(
        &self,
        request: &GetUserRequest,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    fn update_user(
        &self,
        request: &UpdateUserRequest,
    ) -> impl Future<Output = Result<User, UpdateUserError>> + Send;

    fn delete_user(
        &self,
        request: &DeleteUserRequest,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}
