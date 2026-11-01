// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Header writes the `fitsio` crate has no API for: the long-string
//! convention and COMMENT cards. With cfitsio they call it directly, and with
//! the "fitsio-pure" feature they use fitsio-pure's equivalents. Each writes
//! to `hdu`, which must be the file's current HDU for cfitsio.

#[cfg(not(feature = "fitsio-pure"))]
use std::ffi::CString;

use crate::fitsio::{errors::Error, hdu::FitsHdu, FitsFile};

/// Write `LONGSTRN` and the comments that describe the long-string
/// convention, as cfitsio's `ffplsw` does.
#[cfg_attr(not(feature = "fitsio-pure"), allow(unused_variables))]
pub(crate) fn write_longwarn(fptr: &mut FitsFile, hdu: &FitsHdu) -> Result<(), Error> {
    #[cfg(not(feature = "fitsio-pure"))]
    {
        let mut status = 0;
        unsafe {
            // ffplsw = fits_write_key_longwarn
            fitsio_sys::ffplsw(
                fptr.as_raw(), /* I - FITS file pointer  */
                &mut status,   /* IO - error status       */
            );
        }
        crate::fitsio::errors::check_status(status)
    }
    #[cfg(feature = "fitsio-pure")]
    hdu.write_key_longwarn(fptr)
}

/// Write `text` as COMMENT cards, as cfitsio's `ffpcom` does.
#[cfg_attr(not(feature = "fitsio-pure"), allow(unused_variables))]
pub(crate) fn write_comment(fptr: &mut FitsFile, hdu: &FitsHdu, text: &str) -> Result<(), Error> {
    #[cfg(not(feature = "fitsio-pure"))]
    {
        let comm = CString::new(text)?;
        let mut status = 0;
        unsafe {
            // ffpcom = fits_write_comment
            fitsio_sys::ffpcom(fptr.as_raw(), comm.as_ptr(), &mut status);
        }
        crate::fitsio::errors::check_status(status)
    }
    #[cfg(feature = "fitsio-pure")]
    hdu.write_comment(fptr, text)
}

/// Write the string key `key_name`, with CONTINUE cards if it is too long
/// for one card, as cfitsio's `ffpkls` does.
#[cfg_attr(not(feature = "fitsio-pure"), allow(unused_variables))]
pub(crate) fn write_long_string(
    fptr: &mut FitsFile,
    hdu: &FitsHdu,
    key_name: &str,
    value: &str,
    comment: Option<&str>,
) -> Result<(), Error> {
    #[cfg(not(feature = "fitsio-pure"))]
    {
        let key_name = CString::new(key_name)?;
        let value = CString::new(value)?;
        let comment = comment.map(CString::new).transpose()?;
        let mut status = 0;
        unsafe {
            // ffpkls = fits_write_key_longstr
            fitsio_sys::ffpkls(
                fptr.as_raw(),     /* I - FITS file pointer        */
                key_name.as_ptr(), /* I - name of keyword to write */
                value.as_ptr(),    /* I - keyword value            */
                comment.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()), /* I - keyword comment          */
                &mut status, /* IO - error status            */
            );
        }
        crate::fitsio::errors::check_status(status)
    }
    #[cfg(feature = "fitsio-pure")]
    match comment {
        Some(comment) => hdu.write_key(fptr, key_name, (value, comment)),
        None => hdu.write_key(fptr, key_name, value),
    }
}

/// Write the current command-line call as the long string `CMDLINE`. Nothing
/// is written if an argument isn't valid UTF-8 or holds a NUL.
pub(crate) fn write_cmdline(fptr: &mut FitsFile, hdu: &FitsHdu) -> Result<(), Error> {
    match std::env::args_os()
        .map(|a| a.into_string())
        .collect::<Result<Vec<String>, _>>()
    {
        Ok(args) => {
            let value = args.join(" ");
            if value.contains('\0') {
                return Ok(());
            }
            write_long_string(fptr, hdu, "CMDLINE", &value, Some("Command-line call"))
        }
        Err(_) => Ok(()),
    }
}
