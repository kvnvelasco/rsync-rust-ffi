use crate::file::signature::{Signature, SignatureImpl};
use crate::result::RsyncResult;
use crate::stats::RsyncStats;
use crate::utilities::{fopen_with_string, join_to_path_then_string, string_from_path_ref};
use libc::{fclose, FILE};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tempfile::tempdir;

#[link(name = "rsync")]
extern "C" {
    fn rs_delta_file(
        sig: *const SignatureImpl,
        new_file: *const FILE,
        delta_file: *const FILE,
        stats: *mut RsyncStats,
    ) -> RsyncResult;

    fn rs_build_hash_table(rs_signature_t: *mut SignatureImpl);
}

#[derive(Debug)]
pub struct Delta {
    delta: Vec<u8>, // binary contents of a delta file
}

impl<'a> From<&'a Delta> for &'a [u8] {
    fn from(delta: &'a Delta) -> Self {
        delta.delta.as_slice()
    }
}

impl Delta {
    pub fn delta<T: AsRef<Path>>(
        external_file_signature: &mut Signature,
        target_file: T,
    ) -> (Self, RsyncStats) {
        let temp = tempdir().expect("Unable to create working directory");
        let output_file_path = join_to_path_then_string(&temp.path(), "delta_file");
        let target_file_path = string_from_path_ref(&target_file);

        let stats = unsafe {
            let target_file = fopen_with_string(&target_file_path, "rb");
            let output_file = fopen_with_string(&output_file_path, "wb+");

            let mut stats = RsyncStats::new();
            rs_build_hash_table(external_file_signature.as_sig_mut_ptr());

            match rs_delta_file(
                external_file_signature.as_sig_mut_ptr() as *const SignatureImpl,
                target_file,
                output_file,
                &mut stats as *mut RsyncStats,
            ) {
                RsyncResult::Done => {}
                _ => {}
            };
            fclose(output_file);
            fclose(target_file);
            stats
        };

        let mut file = File::open(&output_file_path).expect("Unable to open output file");
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .expect("Unable to read delta file");

        (Delta { delta: contents }, stats)
    }
}

#[cfg(test)]
mod tests {
    use crate::file::delta::Delta;
    use crate::file::signature::Signature;
    use std::borrow::Borrow;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn create_delta_from_signature_and_file() {
        let tempdir = tempfile::tempdir().expect("Unable to create test tempdir");
        let path = tempdir.path().join("tempfile.txt");
        let new_path = tempdir.path().join("newfile.txt");

        {
            let mut temp = File::create(&path).expect("Unable to create tempfile");
            temp.write("A new file".as_bytes())
                .expect("Unable to write tempfile");
        }

        let (mut signature, _stats) = Signature::new(&path).expect("Unable to create signature");

        {
            let mut temp = File::create(&new_path).expect("Unable to create tempfile");
            temp.write(" a new Fail".as_bytes())
                .expect("Unable to write tempfile");
        }

        let (_delta, _stats) = Delta::delta(&mut signature, &new_path);

        dbg!(String::from_utf8_lossy(_delta.borrow().into()));
    }
}
