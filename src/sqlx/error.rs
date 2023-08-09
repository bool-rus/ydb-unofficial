use sqlx_core::error::DatabaseError;
use std::error::Error as StdError;
use crate::error::YdbError;
use sqlx_core::error::ErrorKind as XErrorKind;

impl DatabaseError for YdbError {
    fn message(&self) -> &str {
        self.description()
    }

    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        self
    }

    fn kind(&self) -> XErrorKind {
        XErrorKind::Other
    }
}
