use std::io;

use super::{
    as_handle::AsHandle, diagnostics::Record as DiagnosticRecord, logging::log_diagnostics,
};
use odbc_sys::SqlReturn;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
/// Error type used to indicate a low level ODBC call returned with SQL_ERROR.
pub enum Error {
    /// No Diagnostics available. This is usually the case if allocation of the ODBC Environment
    /// itself fails. In that case no object exist to obtain the diagnostic record from.
    #[error("No Diagnostics available.")]
    NoDiagnostics,
    #[error("{0}")]
    /// SQL Error had been returned by a low level ODBC function call. A Diagnostic record is
    /// obtained and associated with this error.
    Diagnostics(DiagnosticRecord),
    /// A user dialog to complete the connection string has been aborted.
    #[error("The dialog shown to provide or complete the connection string has been aborted.")]
    AbortedConnectionStringCompletion,
    #[error(
        "ODBC diver manager does not seem to support the required ODBC version 3.80. (Most
        likely you need to update unixODBC if you run on a Linux. Diagnostic record returned by
        SQLSetEnvAttr:\n{0}"
    )]
    OdbcApiVersionUnsupported(DiagnosticRecord),
    #[error("Sending data to the database at statement execution time failed. IO error:\n{0}")]
    FailedReadingInput(io::Error),
}

pub trait IntoResult {
    fn into_result(self, handle: &dyn AsHandle) -> Result<(), Error>;
}

impl IntoResult for SqlReturn {
    fn into_result(self: SqlReturn, handle: &dyn AsHandle) -> Result<(), Error> {
        match self {
            // The function has been executed successfully. Holds result.
            SqlReturn::SUCCESS => Ok(()),
            // The function has been executed successfully. There have been warnings. Holds result.
            SqlReturn::SUCCESS_WITH_INFO => {
                log_diagnostics(handle);
                Ok(())
            }
            SqlReturn::ERROR => {
                let mut rec = DiagnosticRecord::default();
                if rec.fill_from(handle, 1) {
                    log_diagnostics(handle);
                    Err(Error::Diagnostics(rec))
                } else {
                    Err(Error::NoDiagnostics)
                }
            }
            r => panic!("Unexpected odbc function result: {:?}", r),
        }
    }
}
