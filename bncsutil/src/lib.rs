extern crate bncsutil_sys as bncsutil;
extern crate libc;

use bytes::{BufMut, BytesMut};
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;

mod nls;

pub fn version() -> u64 {
    unsafe { bncsutil::bncsutil_getVersion() }
}

pub fn version_string() -> String {
    unsafe {
        let mut exe_info_vec: Vec<i8> = vec![0i8; 1024];
        let exe_info_slice = exe_info_vec.as_mut_slice();
        let exe_info_ptr = exe_info_slice.as_mut_ptr();
        let length = bncsutil::bncsutil_getVersionString(exe_info_ptr);
        let exe_info_string =
            String::from_utf8(exe_info_slice.iter().map(|&c| c as u8).collect()).unwrap();
        let exe_info: String = exe_info_string.chars().take(length as usize).collect();

        exe_info
    }
}

pub fn get_exe_info(path: &Path) -> (i32, String, u32) {
    unsafe {
        let path_str = path.to_str().unwrap();

        let s = CString::new(path_str).unwrap();
        let ptr = s.as_ptr();
        let mut exe_version: u32 = 0;
        let mut exe_info_vec: Vec<i8> = vec![0i8; 1024];
        let exe_info_slice = exe_info_vec.as_mut_slice();
        let exe_info_ptr = exe_info_slice.as_mut_ptr();
        let length =
            bncsutil::getExeInfo(ptr, exe_info_ptr, 1024 as usize, &mut exe_version, 1 as i32);
        let exe_info_string =
            String::from_utf8(exe_info_slice.iter().map(|&c| c as u8).collect()).unwrap();
        let exe_info: String = exe_info_string.chars().take(length as usize).collect();

        (length, exe_info, exe_version)
    }
}

pub fn check_revision(value: String, files: Vec<&Path>, mpq_number: i32) -> u32 {
    unsafe {
        let files_str = files
            .iter()
            .map(|val| CString::new(val.to_str().unwrap()).unwrap())
            .collect::<Vec<CString>>();
        let mut files_ptr = files_str
            .iter() // do NOT into_iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();

        let value_cstr = CString::new(value).unwrap();

        let mut result: u64 = 0;

        let error_code = bncsutil::checkRevision(
            value_cstr.as_ptr(),
            files_ptr.as_mut_ptr(),
            files_str.len() as i32,
            mpq_number,
            &mut result,
        );

        println!("result {:?}", result as u32);

        result as u32
    }
}

pub fn check_revision_flat(
    value: String,
    file1: &Path,
    file2: &Path,
    file3: &Path,
    mpq_number: i32,
) -> u32 {
    unsafe {
        let file1_str = CString::new(file1.to_str().unwrap()).unwrap();
        let file2_str = CString::new(file2.to_str().unwrap()).unwrap();
        let file3_str = CString::new(file3.to_str().unwrap()).unwrap();

        let value_cstr = CString::new(value).unwrap();
        let mut result: u64 = 0;

        let error_code = bncsutil::checkRevisionFlat(
            value_cstr.as_ptr(),
            file1_str.as_ptr(),
            file2_str.as_ptr(),
            file3_str.as_ptr(),
            mpq_number,
            &mut result,
        );

        result as u32
    }
}

// { CDKey: 'FFFFFFFFFFFFFFFFFFFFFFFFFF', clientToken: 130744796, serverToken: 1655005115 } { publicValue: 10992493, product: 5650, hash: '0 0 0 0 0 0 0 0' }
pub fn keydecode_quick(
    cd_key: String,
    client_token: u32,
    server_token: u32,
) -> (u32, u32, Vec<u8>) {
    unsafe {
        if cd_key.len() != 26 {
            panic!("Invalid Warcraft 3 Key provided");
        }

        let cd_key_str = CString::new(cd_key).unwrap();
        let mut public_value = 0 as u32;
        let mut product = 0 as u32;
        let mut hash_buf = [0i8; 20];

        /*
           MEXP(int) kd_quick(
               const char* cd_key,
               uint32_t client_token,
               uint32_t server_token,
               uint32_t* public_value,
               uint32_t* product,
               char* hash_buffer,
               size_t buffer_len
           )
        */
        let status = bncsutil::kd_quick(
            cd_key_str.as_ptr(),
            client_token,
            server_token,
            &mut public_value,
            &mut product,
            hash_buf.as_mut_ptr(),
            hash_buf.len(),
        );

        if status == 0 {
            panic!("Failed to kd_quick")
        }

        (
            public_value.clone(),
            product.clone(),
            Vec::from(hash_buf.map(|c| c as u8)),
        )
    }
}

pub fn create_key_info(cd_key: String, client_token: u32, server_token: u32) -> Vec<u8> {
    let keylen = cd_key.len().clone() as u32;
    let (public_value, product, hash) = keydecode_quick(cd_key, client_token, server_token);

    let mut b = BytesMut::new();

    b.put_u32(keylen);
    b.put_u32(product);
    b.put_u32(public_value);
    b.put(&b"\x00\x00\x00\x00"[..]);
    b.put(&hash[..]);

    b.to_vec()
}

pub fn pvpgn_password_hash(password: String) -> Vec<u8> {
    unsafe {
        let mut out_buf = [0i8; 20];
        let pass = CString::new(password).unwrap();

        bncsutil::hashPassword(pass.as_ptr(), out_buf.as_mut_ptr());

        dbg!(pass);

        Vec::from(out_buf.map(|c| c as u8))
    }
}
#[cfg(test)]
mod bncs_tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), 10300);
    }

    #[test]
    fn test_version_string() {
        assert_eq!(version_string(), "1.3.0");
    }

    #[test]
    fn test_get_exe_info() {
        let war3 = Path::new("../mock/war3.exe");

        let info = "war3.exe 02/02/24 12:39:34 562152".to_string();

        assert_eq!(get_exe_info(war3), (33 as i32, info, 18613504 as u32));
    }

    #[test]
    fn test_check_revision_flat() {
        let value = "B=454282227 C=2370009462 A=2264812340 4 A=A^S B=B-C C=C-A A=A+B".to_string();
        let file1 = Path::new("../mock/war3.exe");
        let file2 = Path::new("../mock/Storm.dll");
        let file3 = Path::new("../mock/game.dll");

        assert_eq!(
            check_revision_flat(value, file1, file2, file3, 1),
            2392268693 as u32
        )
    }

    #[test]
    fn test_check_revision() {
        let value = "B=454282227 C=2370009462 A=2264812340 4 A=A^S B=B-C C=C-A A=A+B".to_string();
        let file1 = Path::new("../mock/war3.exe");
        let files = vec![file1];

        assert_eq!(check_revision(value, files, 1), 1397123850 as u32)
    }

    // {
    // CDKey: 'FFFFFFFFFFFFFFFFFFFFFFFFFF',
    // clientToken: 130744796,
    // serverToken: 2115470359 } {
    // publicValue: 10992493,
    // product: 5650, hash: '81 78 135 115 190 107 211 30 62 86 64 112 162 230 136 132 198 76 8 165
    #[test]
    fn test_keydecode_quick() {
        let cd_key = "FFFFFFFFFFFFFFFFFFFFFFFFFF".to_string();
        let client_token: u32 = 130744796;
        let server_token: u32 = 2115470359;
        let result_vec: Vec<u8> = vec![
            16, 95, 106, 232, 69, 15, 81, 141, 27, 2, 250, 43, 67, 21, 89, 120, 196, 223, 45, 222,
        ];
        //        let result_hash = String::from("0 0 0 0 0 0 0 0");
        let (public_value, product, hash) = keydecode_quick(cd_key, client_token, server_token);

        assert_eq!(public_value, 27769709 as u32);
        assert_eq!(product, 5650 as u32);
        assert_eq!(hash, result_vec);
    }

    #[test]
    fn test_create_key_info() {
        let cd_key = "FFFFFFFFFFFFFFFFFFFFFFFFFF".to_string();
        let client_token: u32 = 130744796;
        let server_token: u32 = 2115470359;

        let key_info = create_key_info(cd_key, client_token, server_token);

        assert_eq!(
            key_info,
            vec![
                0, 0, 0, 26, 0, 0, 22, 18, 1, 167, 187, 109, 0, 0, 0, 0, 16, 95, 106, 232, 69, 15,
                81, 141, 27, 2, 250, 43, 67, 21, 89, 120, 196, 223, 45, 222
            ]
        )
    }

    #[test]
    fn test_pvpgn_password_hash() {
        assert_eq!(
            pvpgn_password_hash("pass".to_string()),
            vec![
                110, 200, 23, 20, 119, 166, 65, 4, 164, 164, 111, 5, 24, 230, 149, 79, 255, 64,
                104, 62
            ]
        )
    }
}
