use std::future::Future;

use crate::domain::user::models::{
    CreateUserError, CreateUserRequest, DeleteUserError, DeleteUserRequest, GetUserError,
    GetUserRequest, UpdateUserError, UpdateUserRequest, User,
};

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        req: &CreateUserRequest,
    ) -> impl Future<Output = Result<User, CreateUserError>> + Send;

    fn get_user(
        &self,
        req: &GetUserRequest,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    fn update_user(
        &self,
        req: &UpdateUserRequest,
    ) -> impl Future<Output = Result<User, UpdateUserError>> + Send;

    fn delete_user(
        &self,
        req: &DeleteUserRequest,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}

pub trait UserService: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        req: &CreateUserRequest,
    ) -> impl Future<Output = Result<User, CreateUserError>> + Send;

    fn get_user(
        &self,
        req: &GetUserRequest,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    fn update_user(
        &self,
        req: &UpdateUserRequest,
    ) -> impl Future<Output = Result<User, UpdateUserError>> + Send;

    fn delete_user(
        &self,
        req: &DeleteUserRequest,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}
