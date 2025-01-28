use crate::cli::Args;
use crate::uploads::env::{
    AWS_ACCESS_KEY_ID, AWS_ENDPOINT_URL, AWS_REGION, AWS_SECRET_ACCESS_KEY, S3_FORCE_PATH_STYLE,
};
use crate::warn_if_env_var_is_missing;

pub mod consts;
pub mod env;
pub mod handlers;
pub mod img;
pub mod requests;
pub mod responses;
pub mod s3;

pub fn init(_args: &Args) {
    warn_if_env_var_is_missing!(AWS_ACCESS_KEY_ID, "Image uploads won't work.");
    warn_if_env_var_is_missing!(AWS_SECRET_ACCESS_KEY, "Image uploads won't work.");
    warn_if_env_var_is_missing!(AWS_REGION, "Image uploads won't work.");
    warn_if_env_var_is_missing!(AWS_ENDPOINT_URL, "Will use global AWS S3.");
    warn_if_env_var_is_missing!(S3_FORCE_PATH_STYLE, "Will use global AWS S3.");
}
