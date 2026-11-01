// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Helper functions for reading FITS files.

mod error;

pub(crate) use error::FitsError;

use std::fmt::Display;
#[cfg(not(feature = "fitsio-pure"))]
use std::{
    ffi::{CStr, CString},
    ptr,
};

use crate::fitsio::{
    hdu::*,
    tables::{ConcreteColumnDescription, ReadsCol},
    FitsFile,
};

/// Open a fits file.
#[track_caller]
pub(crate) fn fits_open<P: AsRef<std::path::Path>>(file: P) -> Result<FitsFile, FitsError> {
    FitsFile::open(file.as_ref()).map_err(|e| {
        let caller = std::panic::Location::caller();
        FitsError::Open {
            fits_error: Box::new(e),
            fits_filename: file.as_ref().to_path_buf().into_boxed_path(),
            source_file: caller.file(),
            source_line: caller.line(),
            source_column: caller.column(),
        }
    })
}

/// Open a fits file's HDU.
#[track_caller]
pub(crate) fn fits_open_hdu<T: DescribesHdu + Display + Copy>(
    fits_fptr: &mut FitsFile,
    hdu_description: T,
) -> Result<FitsHdu, FitsError> {
    fits_fptr.hdu(hdu_description).map_err(|e| {
        let caller = std::panic::Location::caller();
        FitsError::Fitsio {
            fits_error: Box::new(e),
            fits_filename: fits_fptr.file_path().into(),
            hdu_description: format!("{hdu_description}").into_boxed_str(),
            source_file: caller.file(),
            source_line: caller.line(),
            source_column: caller.column(),
        }
    })
}

/// Given a FITS file pointer, a HDU that belongs to it, and a keyword that may
/// or may not exist, pull out the value of the keyword, parsing it into the
/// desired type.
#[track_caller]
pub(crate) fn fits_get_optional_key<T: std::str::FromStr>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
) -> Result<Option<T>, FitsError> {
    let unparsed_value: String = match hdu.read_key(fits_fptr, keyword) {
        Ok(key_value) => key_value,
        Err(e) => match &e {
            crate::fitsio::errors::Error::Fits(fe) => match fe.status {
                202 | 204 => return Ok(None),
                _ => {
                    let caller = std::panic::Location::caller();
                    return Err(FitsError::Fitsio {
                        fits_error: Box::new(e),
                        fits_filename: fits_fptr.file_path().into(),
                        hdu_description: format!("{}", hdu.number + 1).into_boxed_str(),
                        source_file: caller.file(),
                        source_line: caller.line(),
                        source_column: caller.column(),
                    });
                }
            },
            _ => {
                let caller = std::panic::Location::caller();
                return Err(FitsError::Fitsio {
                    fits_error: Box::new(e),
                    fits_filename: fits_fptr.file_path().into(),
                    hdu_description: format!("{}", hdu.number + 1).into_boxed_str(),
                    source_file: caller.file(),
                    source_line: caller.line(),
                    source_column: caller.column(),
                });
            }
        },
    };

    match unparsed_value.parse() {
        Ok(parsed_value) => Ok(Some(parsed_value)),
        Err(_) => {
            let caller = std::panic::Location::caller();
            Err(FitsError::Parse {
                key: keyword.to_string().into_boxed_str(),
                fits_filename: fits_fptr.file_path().into(),
                hdu_num: hdu.number + 1,
                source_file: caller.file(),
                source_line: caller.line(),
                source_column: caller.column(),
            })
        }
    }
}

/// Given a FITS file pointer, a HDU that belongs to it, and a keyword, pull out
/// the value of the keyword, parsing it into the desired type.
#[track_caller]
pub(crate) fn fits_get_required_key<T: std::str::FromStr>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
) -> Result<T, FitsError> {
    match fits_get_optional_key(fits_fptr, hdu, keyword) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => {
            let caller = std::panic::Location::caller();
            Err(FitsError::MissingKey {
                key: keyword.to_string().into_boxed_str(),
                fits_filename: fits_fptr.file_path().into(),
                hdu_num: hdu.number + 1,
                source_file: caller.file(),
                source_line: caller.line(),
                source_column: caller.column(),
            })
        }
        Err(error) => Err(error),
    }
}

/// Get a column from a fits file's HDU.
#[track_caller]
pub(crate) fn fits_get_col<T: crate::fitsio::tables::ReadsCol>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
) -> Result<Vec<T>, FitsError> {
    hdu.read_col(fits_fptr, keyword).map_err(|e| {
        let caller = std::panic::Location::caller();
        FitsError::Fitsio {
            fits_error: Box::new(e),
            fits_filename: fits_fptr.file_path().into(),
            hdu_description: format!("{}", hdu.number + 1).into_boxed_str(),
            source_file: caller.file(),
            source_line: caller.line(),
            source_column: caller.column(),
        }
    })
}

/// Given a FITS file pointer, and a keyword to a long string keyword that may
/// or may not exist, pull out the long string of the keyword. This deals with
/// FITSs CONTINUE mechanism by calling a low level fits function.
#[track_caller]
pub(crate) fn fits_get_optional_key_long_string(
    fits_fptr: &mut crate::fitsio::FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
) -> Result<Option<String>, FitsError> {
    // Read the long string.
    #[cfg(not(feature = "fitsio-pure"))]
    let keyword_ffi = CString::new(keyword)
        .expect("fits_get_optional_fits_long_string: CString::new() failed for keyword");
    #[cfg(not(feature = "fitsio-pure"))]
    let long_string = unsafe {
        let mut status = 0;
        let mut long_string_ptr = ptr::null_mut();
        // ffgkls = fits_read_key_longstr
        fitsio_sys::ffgkls(
            fits_fptr.as_raw(),
            keyword_ffi.as_ptr(),
            &mut long_string_ptr,
            ptr::null_mut(),
            &mut status,
        );
        // Check the call worked!
        match status {
            0 => {
                let long_string = CStr::from_ptr(long_string_ptr)
                    .to_str()
                    .expect("fits_get_optional_key_long_string: reading C string as UTF-8 failed")
                    .to_string();
                // Free the cfitsio-allocated string. The status code passed
                // isn't useful (have a look at the source if you don't believe
                // me!)
                // fffree = fits_free_memory
                fitsio_sys::fffree(long_string_ptr.cast(), &mut 0);
                Some(long_string)
            }
            202 | 204 => None,
            _ => {
                let caller = std::panic::Location::caller();
                return Err(FitsError::LongString {
                    key: keyword.to_string().into_boxed_str(),
                    fits_filename: fits_fptr.file_path().into(),
                    hdu_num: hdu.number + 1,
                    source_file: caller.file(),
                    source_line: caller.line(),
                    source_column: caller.column(),
                });
            }
        }
    };
    // read_key_longstr follows CONTINUE cards, as ffgkls does.
    #[cfg(feature = "fitsio-pure")]
    let long_string = match hdu.read_key_longstr::<String>(fits_fptr, keyword) {
        Ok(long_string) => Some(long_string),
        Err(crate::fitsio::errors::Error::Fits(e)) if e.status == 202 || e.status == 204 => None,
        Err(_) => {
            let caller = std::panic::Location::caller();
            return Err(FitsError::LongString {
                key: keyword.to_string().into_boxed_str(),
                fits_filename: fits_fptr.file_path().into(),
                hdu_num: hdu.number + 1,
                source_file: caller.file(),
                source_line: caller.line(),
                source_column: caller.column(),
            });
        }
    };

    Ok(long_string)
}

/// Given a FITS file pointer, and a keyword to a long string keyword, pull out
/// the long string of the keyword. This deals with FITSs CONTINUE mechanism by
/// calling a low level fits function.
#[track_caller]
pub(crate) fn _fits_get_required_key_long_string(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
) -> Result<String, FitsError> {
    match fits_get_optional_key_long_string(fits_fptr, hdu, keyword) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => {
            let caller = std::panic::Location::caller();
            Err(FitsError::MissingKey {
                key: keyword.to_string().into_boxed_str(),
                fits_filename: fits_fptr.file_path().into(),
                hdu_num: hdu.number + 1,
                source_file: caller.file(),
                source_line: caller.line(),
                source_column: caller.column(),
            })
        }
        Err(error) => Err(error),
    }
}

/// Get the size of the image on the supplied FITS file pointer and HDU.
#[track_caller]
pub(crate) fn _fits_get_image_size<'a>(
    fits_fptr: &FitsFile,
    hdu: &'a FitsHdu,
) -> Result<&'a Vec<usize>, FitsError> {
    match &hdu.info {
        HduInfo::ImageInfo { shape, .. } => Ok(shape),
        _ => {
            let caller = std::panic::Location::caller();
            Err(FitsError::NotImage {
                fits_filename: fits_fptr.file_path().into(),
                hdu_num: hdu.number + 1,
                source_file: caller.file(),
                source_line: caller.line(),
                source_column: caller.column(),
            })
        }
    }
}

/// Given a FITS file pointer and a HDU, read the associated image.
#[track_caller]
pub(crate) fn fits_get_image<T: crate::fitsio::images::ReadImage>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
) -> Result<T, FitsError> {
    match &hdu.info {
        HduInfo::ImageInfo { .. } => hdu.read_image(fits_fptr).map_err(|e| {
            let caller = std::panic::Location::caller();
            FitsError::Fitsio {
                fits_error: Box::new(e),
                fits_filename: fits_fptr.file_path().into(),
                hdu_description: format!("{}", hdu.number + 1).into_boxed_str(),
                source_file: caller.file(),
                source_line: caller.line(),
                source_column: caller.column(),
            }
        }),
        _ => {
            let caller = std::panic::Location::caller();
            Err(FitsError::NotImage {
                fits_filename: fits_fptr.file_path().into(),
                hdu_num: hdu.number + 1,
                source_file: caller.file(),
                source_line: caller.line(),
                source_column: caller.column(),
            })
        }
    }
}

/// Given a FITS file pointer and a HDU, read the associated float image.
#[track_caller]
pub(crate) fn _fits_get_float_image_into_buffer(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    buffer: &mut [f32],
) -> Result<(), FitsError> {
    #[cfg(feature = "fitsio-pure")]
    {
        use crate::fitsio::images::ReadImageIntoBuffer;

        // Like ffgpv: the first `buffer.len()` pixels, BSCALE/BZERO applied.
        if let Err(e) = f32::read_image_into_buffer(fits_fptr, hdu, buffer) {
            let caller = std::panic::Location::caller();
            return Err(FitsError::Fitsio {
                fits_error: Box::new(e),
                fits_filename: fits_fptr.file_path().into(),
                hdu_description: format!("{}", hdu.number + 1).into_boxed_str(),
                source_file: caller.file(),
                source_line: caller.line(),
                source_column: caller.column(),
            });
        }
    }

    #[cfg(not(feature = "fitsio-pure"))]
    unsafe {
        // Get raw ptr and length to user supplied buffer
        let buffer_len = buffer.len() as i64;
        let buffer_ptr = buffer.as_mut_ptr();

        // Call the underlying cfitsio read function for floats
        let mut status = 0;
        // ffgpv = fits_read_img
        fitsio_sys::ffgpv(
            fits_fptr.as_raw(),
            fitsio_sys::TFLOAT as _,
            1,
            buffer_len,
            ptr::null_mut(),
            buffer_ptr as *mut _,
            ptr::null_mut(),
            &mut status,
        );

        // Check fits call status
        match crate::fitsio::errors::check_status(status) {
            Ok(_) => {}
            Err(e) => {
                let caller = std::panic::Location::caller();
                return Err(FitsError::Fitsio {
                    fits_error: Box::new(e),
                    fits_filename: fits_fptr.file_path().into(),
                    hdu_description: format!("{}", hdu.number + 1).into_boxed_str(),
                    source_file: caller.file(),
                    source_line: caller.line(),
                    source_column: caller.column(),
                });
            }
        }
    }

    Ok(())
}

/// Pull out fits array-in-a-cell values; useful for e.g. STABXYZ. This function
/// must be pointed at data containing f64s (i.e. TDOUBLE).
#[track_caller]
pub(crate) fn fits_read_cell_f64_array<const NUM_ELEM: usize>(
    fits_ptr: &mut FitsFile,
    hdu: &FitsHdu,
    col_name: &'static str,
    row: i64,
) -> Result<[f64; NUM_ELEM], FitsError> {
    let to_fits_error = |err| FitsError::ReadCellArray {
        col_name,
        hdu_num: hdu.number + 1,
        err: Box::new(err),
    };
    // 219 = COL_NOT_FOUND, as ffgcno reports it.
    let col_desc = fits_find_col(hdu, col_name, false)
        .map_err(to_fits_error)?
        .ok_or_else(|| to_fits_error(col_not_found()))?;
    let row = usize::try_from(row).expect("row is not negative");
    let values = fits_read_cell_array::<f64>(fits_ptr, hdu, col_desc, row, NUM_ELEM)
        .map_err(to_fits_error)?;
    let mut array = [0.0; NUM_ELEM];
    array.copy_from_slice(&values);
    Ok(array)
}

/// The error cfitsio gives for a column that doesn't exist (`COL_NOT_FOUND`).
pub(crate) fn col_not_found() -> crate::fitsio::errors::Error {
    crate::fitsio::errors::check_status(219).expect_err("219 is an error status")
}

/// Find the table column `col_name` as cfitsio's `ffgcno` does, matching case
/// or not, and return its description. A name that matches no column is
/// `None`; one that matches more than one is `COL_NOT_UNIQUE` (237).
pub(crate) fn fits_find_col<'a>(
    hdu: &'a FitsHdu,
    col_name: &str,
    case_sensitive: bool,
) -> Result<Option<&'a ConcreteColumnDescription>, crate::fitsio::errors::Error> {
    let HduInfo::TableInfo {
        column_descriptions,
        ..
    } = &hdu.info
    else {
        // NOT_TABLE
        return Err(crate::fitsio::errors::check_status(235).expect_err("235 is an error status"));
    };
    let mut matches = column_descriptions.iter().filter(|desc| {
        if case_sensitive {
            desc.name == col_name
        } else {
            desc.name.eq_ignore_ascii_case(col_name)
        }
    });
    match (matches.next(), matches.next()) {
        (Some(desc), None) => Ok(Some(desc)),
        (None, _) => Ok(None),
        (Some(_), Some(_)) => {
            Err(crate::fitsio::errors::check_status(237).expect_err("237 is an error status"))
        }
    }
}

/// Read `n_elem` values of the column `col_desc` from row `row` (0-indexed)
/// on, as cfitsio's `ffgcv` reads them: a read longer than a row runs on into
/// the following rows.
pub(crate) fn fits_read_cell_array<T: ReadsCol>(
    fits_ptr: &mut FitsFile,
    hdu: &FitsHdu,
    col_desc: &ConcreteColumnDescription,
    row: usize,
    n_elem: usize,
) -> Result<Vec<T>, crate::fitsio::errors::Error> {
    let num_rows = n_elem.div_ceil(col_desc.data_type.repeat.max(1)).max(1);
    let mut values = hdu.read_col_range(fits_ptr, &col_desc.name, &(row..row + num_rows))?;
    values.truncate(n_elem);
    Ok(values)
}
