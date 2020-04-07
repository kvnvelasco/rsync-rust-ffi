use crate::magic_number::MagicNumber;
use crate::result::RsyncResult;
use crate::stats::RsyncStats;
use crate::utilities::{fopen_with_string, join_to_path_then_string, string_from_path_ref};
use libc::{fclose, size_t, FILE};
use std::error::Error;
use std::ffi::c_void;
use std::os::raw::c_int;
use std::path::Path;
use tempfile::tempdir;

#[link(name = "rsync")]
extern "C" {
    fn rs_sig_file(
        old_file: *const FILE,
        sig_file: *const FILE,
        block_len: size_t,
        strong_len: size_t,
        magic_number: MagicNumber,
        stats: *mut RsyncStats,
    ) -> RsyncResult;

    fn rs_loadsig_file(
        sig_file: *const FILE,
        sumset: *const *mut SignatureImpl,
        stats: *mut RsyncStats,
    ) -> RsyncResult;
}

#[derive(Debug)]
pub struct Signature {
    sig: *mut SignatureImpl,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct SignatureImpl {
    magic: MagicNumber,
    block_len: c_int,
    strong_sum_len: c_int,
    count: c_int,
    size: c_int,
    block_sigs: *mut c_void,
    hashtable: *mut c_void,
    calc_strong_count: size_t,
}

impl Default for SignatureImpl {
    fn default() -> Self {
        SignatureImpl {
            magic: MagicNumber::MD5,
            block_len: 0,
            strong_sum_len: 0,
            count: 0,
            size: 0,
            block_sigs: std::ptr::null_mut(),
            hashtable: std::ptr::null_mut(),
            calc_strong_count: 0,
        }
    }
}

impl Signature {
    pub(crate) unsafe fn as_sig_mut_ptr(&mut self) -> *mut SignatureImpl {
        self.sig
    }

    pub fn new<T: AsRef<Path>>(file: T) -> Result<(Self, RsyncStats), Box<dyn Error>> {
        let target_file = string_from_path_ref(&file);

        let tempdir = tempdir().expect("Unable to allocate temporary directory");
        let signature_file_path = join_to_path_then_string(&tempdir.path(), "signature_file");

        let (signature, stats) = unsafe {
            let target_file_descriptor = fopen_with_string(&target_file, "rb");
            let signature_file_descriptor = fopen_with_string(&signature_file_path, "wb+");

            let mut stats = RsyncStats::new();

            match rs_sig_file(
                target_file_descriptor,
                signature_file_descriptor,
                2048 as size_t,
                32 as size_t,
                MagicNumber::Blake2Hash,
                &mut stats as *mut RsyncStats,
            ) {
                RsyncResult::Done => {}
                _ => panic!("Unable to generate signature"),
            }

            // flush the changes to the signature fd
            fclose(target_file_descriptor);
            fclose(signature_file_descriptor);

            let signature_file_descriptor = fopen_with_string(&signature_file_path, "rb");

            let ptr = std::ptr::null_mut() as *mut SignatureImpl;
            match rs_loadsig_file(
                signature_file_descriptor as *const FILE,
                &ptr as *const *mut SignatureImpl,
                std::ptr::null_mut(),
            ) {
                RsyncResult::Done => {}
                _ => panic!("Unable to load signature file"),
            };

            fclose(signature_file_descriptor);
            (Signature { sig: ptr }, stats)
        };

        Ok((signature, stats))
    }
}

#[cfg(test)]
mod tests {
    use super::Signature;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn create_signature_from_file() {
        let tempdir = tempfile::tempdir().expect("Unable to create test tempdir");
        let path = tempdir.path().join("tempfile.txt");

        {
            let mut temp = File::create(&path).expect("Unable to create tempfile");
            temp.write(&[0x23 as u8; 30_000])
                .expect("Unable to write tempfile");
        }

        let _signature = Signature::new(&path).expect("Unable to create signature");
    }
}
