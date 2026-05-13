// SPDX-License-Identifier: GPL-2.0-only
use core::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_long;
use std::sync::OnceLock;

static CURL_INIT: OnceLock<()> = OnceLock::new();

#[repr(C)]
pub struct CURL {
    _private: [u8; 0],
}

type CURLcode = c_int;
type CURLoption = u32;
type CURLINFO = u32;

const CURLE_OK: CURLcode = 0;

const CURL_GLOBAL_DEFAULT: c_long = 3;

const CURLOPT_URL: CURLoption = 10002;
const CURLOPT_NOBODY: CURLoption = 44;
const CURLOPT_TIMEOUT: CURLoption = 13;
const CURLOPT_FOLLOWLOCATION: CURLoption = 52;
const CURLOPT_USERAGENT: CURLoption = 10018;
const CURLOPT_HEADERFUNCTION: CURLoption = 20079;
const CURLOPT_HEADERDATA: CURLoption = 10029;

const CURLINFO_RESPONSE_CODE: CURLINFO = 0x200002;

#[link(name = "curl")]
unsafe extern "C" {}

unsafe extern "C" {
    fn curl_global_init(flags: c_long) -> CURLcode;

    fn curl_easy_init() -> *mut CURL;

    fn curl_easy_cleanup(handle: *mut CURL);

    fn curl_easy_perform(handle: *mut CURL) -> CURLcode;

    fn curl_easy_strerror(code: CURLcode) -> *const c_char;

    fn curl_easy_setopt(handle: *mut CURL, option: CURLoption, ...) -> CURLcode;

    fn curl_easy_getinfo(handle: *mut CURL, info: CURLINFO, ...) -> CURLcode;
}

fn init_curl() {
    CURL_INIT.get_or_init(|| unsafe {
        curl_global_init(CURL_GLOBAL_DEFAULT);
    });
}

fn curl_error(code: CURLcode) -> String {
    unsafe {
        let ptr = curl_easy_strerror(code);

        if ptr.is_null() {
            return format!("curl error {}", code);
        }

        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

struct HeaderCapture {
    location: Option<String>,
}

extern "C" fn header_callback(
    buffer: *mut c_char,
    size: usize,
    nitems: usize,
    userdata: *mut c_void,
) -> usize {
    let total = size * nitems;

    unsafe {
        let bytes = std::slice::from_raw_parts(buffer as *const u8, total);

        if let Ok(header) = std::str::from_utf8(bytes) {
            let lower = header.to_ascii_lowercase();

            if lower.starts_with("location:") {
                if let Some((_, value)) = header.split_once(':') {
                    let capture = &mut *(userdata as *mut HeaderCapture);

                    capture.location = Some(value.trim().to_string());
                }
            }
        }
    }

    total
}

#[derive(Debug)]
pub struct HttpResponse {
    pub response_code: u16,
    pub location: Option<String>,
}

pub fn head_request(url: &str) -> Result<HttpResponse, String> {
    init_curl();

    unsafe {
        let curl = curl_easy_init();

        if curl.is_null() {
            return Err("curl_easy_init failed".into());
        }

        let url_c = match CString::new(url) {
            Ok(v) => v,
            Err(_) => {
                curl_easy_cleanup(curl);

                return Err("invalid URL".into());
            }
        };

        let ua_c = CString::new("link-checker/1.0").unwrap();

        let mut headers = HeaderCapture { location: None };

        macro_rules! setopt {
            ($opt:expr, $val:expr) => {{
                let rc = curl_easy_setopt(curl, $opt, $val);

                if rc != CURLE_OK {
                    curl_easy_cleanup(curl);

                    return Err(curl_error(rc));
                }
            }};
        }

        setopt!(CURLOPT_URL, url_c.as_ptr());
        setopt!(CURLOPT_NOBODY, 1 as c_long);
        setopt!(CURLOPT_TIMEOUT, 10 as c_long);
        setopt!(CURLOPT_FOLLOWLOCATION, 0 as c_long);
        setopt!(CURLOPT_USERAGENT, ua_c.as_ptr());

        setopt!(
            CURLOPT_HEADERFUNCTION,
            header_callback as extern "C" fn(_, _, _, _) -> _
        );

        setopt!(CURLOPT_HEADERDATA, &mut headers as *mut _ as *mut c_void);

        let rc = curl_easy_perform(curl);

        if rc != CURLE_OK {
            curl_easy_cleanup(curl);

            return Err(curl_error(rc));
        }

        let mut response_code: c_long = 0;

        let rc = curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &mut response_code);

        if rc != CURLE_OK {
            curl_easy_cleanup(curl);

            return Err(curl_error(rc));
        }

        curl_easy_cleanup(curl);

        Ok(HttpResponse {
            response_code: response_code as u16,
            location: headers.location,
        })
    }
}
