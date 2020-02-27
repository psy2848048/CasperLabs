use types::ApiError;

#[repr(u16)]
pub enum Error {
    UnknownProxyApi = 1, // 65537
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}
