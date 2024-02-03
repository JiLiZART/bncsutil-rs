extern crate bncsutil_sys as bncsutil;
extern crate libc;

use std::ffi::CString;

#[derive(Debug)]
pub struct NLS {
    n: *mut bncsutil::_nls,
}

impl Drop for NLS {
    fn drop(&mut self) {
        unsafe {
            bncsutil::nls_free(self.n);
        }
    }
}

impl NLS {
    pub fn new(username: String, password: String) -> Self {
        unsafe {
            let name = CString::new(username).unwrap();
            let pass = CString::new(password).unwrap();
            let n = bncsutil::nls_init(name.as_ptr(), pass.as_ptr());

            Self { n }
        }
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        let mut buf = [0i8, 32];

        unsafe {
            bncsutil::nls_get_A(self.n, buf.as_mut_ptr());

            buf.iter().map(|&c| c as u8).collect()
        }
    }

    pub fn get_client_session_key(&self, salt: Vec<u8>, server_key: Vec<u8>) -> Vec<u8> {
        let mut buf = [0i8, 20];

        unsafe {
            let mut server = CString::from_vec_unchecked(server_key);
            let mut s = CString::from_vec_unchecked(salt);

            bncsutil::nls_get_M1(self.n, buf.as_mut_ptr(), server.as_ptr(), s.as_ptr());

            buf.iter().map(|&c| c as u8).collect()
        }
    }
}

#[cfg(test)]
mod nls_tests {
    use super::*;

    #[test]
    fn test_get_public_key() {
        let nls = NLS::new(String::from("test"), String::from("test"));

        assert_eq!(nls.get_public_key(), vec![0]);
    }

    #[test]
    fn test_get_client_session_key() {
        let nls = NLS::new(String::from("test"), String::from("test"));

        assert_eq!(nls.get_client_session_key(vec![0], vec![0]), vec![0])
    }
}
