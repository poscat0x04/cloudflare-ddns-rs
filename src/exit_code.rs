use std::fmt::Debug;
use std::process::{ExitCode, Termination};

pub type Result<T, E = ExitCodeError> = core::result::Result<T, E>;

pub struct ExitCodeError {
    error: Box<dyn Debug + Send + Sync>,
    exit_code: ExitCode,
}

impl Termination for ExitCodeError {
    fn report(self) -> ExitCode {
        eprintln!("Error: {:?}", self.error);
        self.exit_code
    }
}

impl<T> From<T> for ExitCodeError
    where
        T: Debug + Send + Sync + 'static
{
    fn from(value: T) -> Self {
        ExitCodeError {
            error: Box::new(value),
            exit_code: ExitCode::FAILURE,
        }
    }
}

pub trait WithExitCode<T, E> {
    fn exit_code(self, e: ExitCode) -> Result<T, ExitCodeError>;

    fn map_exit_code<F>(self, f: F) -> Result<T, ExitCodeError>
        where
            F: Fn(&E) -> ExitCode;
}

impl<T, E> WithExitCode<T, E> for Result<T, E>
    where
        E: Debug + Send + Sync + 'static
{
    fn exit_code(self, e: ExitCode) -> Result<T, ExitCodeError> {
        self.map_err(|error| ExitCodeError {
            error: Box::new(error),
            exit_code: e,
        })
    }

    fn map_exit_code<F>(self, f: F) -> Result<T, ExitCodeError>
        where F: Fn(&E) -> ExitCode
    {
        self.map_err(|error| ExitCodeError {
            exit_code: f(&error),
            error: Box::new(error),
        })
    }
}

#[repr(transparent)]
pub struct AppResult<T, E>(
    pub Result<T, E>
);

impl<T, E> Termination for AppResult<T, E>
    where
        T: Termination,
        E: Termination,
{
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(a) => Termination::report(a),
            Err(a) => Termination::report(a)
        }
    }
}

#[inline]
pub fn user_error_exitcode() -> ExitCode {
    ExitCode::from(2)
}
