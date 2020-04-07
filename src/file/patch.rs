use crate::file::delta::Delta;
use crate::result::RsyncResult;
use crate::stats::RsyncStats;
use crate::utilities::{fopen_with_string, join_to_path_then_string, string_from_path_ref};
use libc::{fclose, FILE};
use std::borrow::Borrow;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tempfile::tempdir;

#[link(name = "rsync")]
extern "C" {

    fn rs_patch_file(
        basis_file: *const FILE,
        delta_file: *const FILE,
        new_file: *const FILE,
        stats: *mut RsyncStats,
    ) -> RsyncResult;

}

pub fn patch_file<T: AsRef<Path>>(file_to_patch: T, delta: Delta) -> Vec<u8> {
    let temp = tempdir().expect("Unable to create tempdir");
    let file_location = string_from_path_ref(&file_to_patch);
    let temporary_output_location = join_to_path_then_string(&temp.path(), "output");
    let delta_file = string_from_path_ref(&{
        let path = temp.path().join("delta_file");
        let mut file = File::create(&path).expect("Unable to prepare delta file");
        file.write_all(delta.borrow().into());
        path
    });

    unsafe {
        let target_file = fopen_with_string(&file_location, "rb");
        let delta_file = fopen_with_string(&delta_file, "rb");
        let output_file = fopen_with_string(&temporary_output_location, "wb+");

        rs_patch_file(target_file, delta_file, output_file, std::ptr::null_mut());

        fclose(target_file);
        fclose(delta_file);
        fclose(output_file);
    };

    let mut file = File::open(&temporary_output_location).expect("Unable to read output file");
    let mut output = Vec::new();
    file.read_to_end(&mut output);

    output
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::file::signature::Signature;
    use crate::file::delta::Delta;
    use crate::file::patch::patch_file;
    use std::io::Write;

    #[test]
    fn apply_a_delta_to_a_file() {
        let tempdir = tempfile::tempdir().expect("Unable to create test tempdir");
        let path = tempdir.path().join("tempfile.txt");
        let new_path = tempdir.path().join("newfile.txt");

        {
            let mut temp = File::create(&path).expect("Unable to create tempfile");
            temp.write(&[0x23 as u8; 35_000])
                .expect("Unable to write tempfile");
        }

        let (mut signature, _stats) = Signature::new(&path).expect("Unable to create signature");

        {
            let mut temp = File::create(&new_path).expect("Unable to create tempfile");
            temp.write(&[0x67 as u8; 35_000])
                .expect("Unable to write tempfile");
        }

        let (_delta, _stats) = Delta::delta(&mut signature, &new_path);

        let data = patch_file(path, _delta);

        let string_data = String::from_utf8_lossy(data.as_slice());

        assert_eq!(string_data, String::from_utf8_lossy(&[0x67 as u8; 35_000]));
    }
}