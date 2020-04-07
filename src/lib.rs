pub mod file;
mod utilities;

mod magic_number {
    #[repr(C)]
    #[derive(Debug)]
    #[allow(dead_code)]
    pub enum MagicNumber {
        Blake2Hash = 0x72730137,
        Delta = 0x72730236,
        MD5 = 0x72730136,
    }
}

mod result {
    #[allow(dead_code)]
    #[repr(C)]
    pub enum RsyncResult {
        Done = 0,
        Blocked = 1,
        Running = 2,
        Skipped = 77,
        IOError = 100,
        SyntaxError = 101,
        // Out of memory
        MemoryError = 102,
        InputEndedError = 103,
        //  Bad magic number at start of stream.  Probably not a librsync file,
        // or possibly the wrong kind of file or from an incompatible library version.
        BadMagicError = 104,
        UnimplementedError = 105,
        Corrupt = 106,
        InternalError = 107,
        ParamError = 108,
    }
}

mod stats {
    use libc::size_t;
    use std::os::raw::{c_char, c_int, c_longlong};

    #[repr(C)]
    #[derive(Debug)]
    pub struct RsyncStats {
        pub op: *mut c_char,
        lit_cmds: c_int,
        lit_bytes: c_longlong,
        lit_cmdbytes: c_longlong,
        copy_cmds: c_longlong,
        copy_bytes: c_longlong,
        copy_cmdbytes: c_longlong,
        sig_cmds: c_longlong,
        sig_bytes: c_longlong,
        false_matches: c_int,
        sig_blocks: c_longlong,
        block_len: size_t,
        in_bytes: c_longlong,
        out_bytes: c_longlong,
    }
    impl RsyncStats {
        pub fn new() -> Self {
            RsyncStats {
                op: std::ptr::null_mut(),
                lit_cmds: 0,
                lit_bytes: 0,
                lit_cmdbytes: 0,
                copy_cmds: 0,
                copy_bytes: 0,
                copy_cmdbytes: 0,
                sig_cmds: 0,
                sig_bytes: 0,
                false_matches: 0,
                sig_blocks: 0,
                block_len: 0,
                in_bytes: 0,
                out_bytes: 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
