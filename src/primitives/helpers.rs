use std::{ffi::c_void, mem::ManuallyDrop, ptr};
use windows::{
    core::BSTR,
    Win32::System::{
        Com::{
            SAFEARRAY, SAFEARRAYBOUND, VARENUM, VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0,
            VT_ARRAY, VT_BSTR, VT_UI1, VT_VARIANT,
        },
        Ole::{
            SafeArrayAccessData, SafeArrayCreate, SafeArrayCreateVector, SafeArrayGetUBound,
            SafeArrayPutElement, SafeArrayUnaccessData,
        },
    },
};

pub fn prepare_assembly(bytes: &[u8]) -> Result<*mut SAFEARRAY, String> {
    let mut bounds = SAFEARRAYBOUND {
        cElements: bytes.len() as _,
        lLbound: 0,
    };

    let safe_array_ptr: *mut SAFEARRAY = unsafe { SafeArrayCreate(VT_UI1, 1, &mut bounds) };
    let mut pv_data: *mut c_void = ptr::null_mut();

    match unsafe { SafeArrayAccessData(safe_array_ptr, &mut pv_data) } {
        Ok(_) => {},
        Err(e) => {
            return Err(format!(
                "Could not prepare assembly due to a safe array related error: {:?}",
                e.code()
            ))
        },
    }

    unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), pv_data.cast(), bytes.len()) };

    match unsafe { SafeArrayUnaccessData(safe_array_ptr) } {
        Ok(_) => {},
        Err(e) => {
            return Err(format!(
                "Could not prepare assembly due to a safe array related error: {:?}",
                e.code()
            ))
        },
    };

    Ok(safe_array_ptr)
}

pub fn get_array_length(array_ptr: *mut SAFEARRAY) -> i32 {
    unsafe { SafeArrayGetUBound(array_ptr, 1) }.unwrap_or(0)
}

pub fn empty_array() -> *mut SAFEARRAY {
    unsafe { SafeArrayCreateVector(VT_VARIANT, 0, 0) }
}

pub fn wrap_string_in_variant(string: &str) -> VARIANT {
    let inner = BSTR::from(string);

    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_BSTR,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 {
                    bstrVal: ManuallyDrop::new(inner),
                },
            }),
        },
    }
}

pub fn wrap_strings_in_array(strings: &[String]) -> Result<VARIANT, String> {
    let mut inner = vec![];

    for string in strings.iter() {
        inner.push(BSTR::from(string).into_raw())
    }

    let safe_array_ptr: *mut SAFEARRAY =
        unsafe { SafeArrayCreateVector(VT_BSTR, 0, inner.len() as u32) };

    for i in 0..inner.len() {
        let indices: [i32; 1] = [i as _];
        let v_ref = &inner[i];
        match unsafe { SafeArrayPutElement(safe_array_ptr, indices.as_ptr(), *v_ref as *const _) } {
            Ok(_) => {},
            Err(e) => {
                return Err(format!(
                    "Could not create an array of strings: {:?}",
                    e.code()
                ))
            },
        }
    }

    Ok(VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: VARENUM(VT_BSTR.0 | VT_ARRAY.0),
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 {
                    parray: safe_array_ptr,
                },
            }),
        },
    })
}

pub fn wrap_method_arguments(arguments: Vec<VARIANT>) -> Result<*mut SAFEARRAY, String> {
    let variant_array_ptr: *mut SAFEARRAY =
        unsafe { SafeArrayCreateVector(VT_VARIANT, 0, arguments.len() as u32) };

    for i in 0..arguments.len() {
        let indices: [i32; 1] = [i as _];
        let v_ref: *const _ = &arguments[i];
        match unsafe { SafeArrayPutElement(variant_array_ptr, indices.as_ptr(), v_ref as *const _) }
        {
            Ok(_) => {},
            Err(e) => {
                return Err(format!(
                    "Could not create an array of arguments: {:?}",
                    e.code()
                ))
            },
        }
    }

    Ok(variant_array_ptr)
}
