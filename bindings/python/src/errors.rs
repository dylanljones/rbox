// Author: Dylan Jones
// Date:   2025-05-12

use pyo3::create_exception;
use pyo3::exceptions::PyException;

create_exception!(
    rbox,
    Error,
    PyException,
    "The base class of the other exceptions in this module. Use this to catch all errors with one single except statement"
);

create_exception!(
    rbox,
    DatabaseError,
    Error,
    "Exception raised for errors that are related to the master.db database."
);

create_exception!(
    rbox,
    AnlzError,
    Error,
    "Exception raised for errors that are related to the ANLZ files."
);

create_exception!(
    rbox,
    XmlError,
    Error,
    "Exception raised for errors that are related to the Rekordbox XML handler."
);

create_exception!(
    rbox,
    SettingError,
    Error,
    "Exception raised for errors that are related to the Rekordbox MySetting handler."
);
