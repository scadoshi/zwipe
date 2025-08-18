use std::future::Future;

use crate::domain::user::models::{
    CreateUserRequest, DeleteUserError, DeleteUserRequest, GetUserError, GetUserRequest,
    UpdateUserRequest, User, UserError,
};

pub trait UserRepository: Send + Sync + Clone + 'static {
    pub fn create_user(
        &self,
        req: &CreateUserRequest,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    pub fn get_user(
        &self,
        req: &GetUserRequest,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    pub fn update_user(
        &self,
        req: &UpdateUserRequest,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    pub fn delete_user(
        &self,
        req: &DeleteUserRequest,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}

pub trait UserService: Send + Sync + Clone + 'static {
    pub fn create_user(
        &self,
        req: &CreateUserRequest,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    pub fn get_user(
        &self,
        req: &GetUserRequest,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    pub fn update_user(
        &self,
        req: &UpdateUserRequest,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    pub fn delete_user(
        &self,
        req: &DeleteUserRequest,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}
