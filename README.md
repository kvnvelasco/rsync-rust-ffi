# Rust FFI bindings for librsync2 
Dynamically linked to librsync2 / librsync-dev

## Features 
- [x] Whole File API
    - [x] Generate Signature 
    - [x] Load signature into memory 
    - [x] Create Delta 
    - [x] Apply delta onto file 
- [ ] Streaming API 
- [ ] Error handling

## Usage 

### File API 

#### Generating a Signature 
Signatures are stored in memory even though the whole file api is used. Temporary directories are utilized 
to hold values until they get initialized in memory. 

```
let path = PathBuf::from("some/file/location");
let (signature, stats) = Signature::new(&path).unwrap;
```

#### Generating a delta
Delta files are a `Vec<u8>` representations of rsync delta files. A delta takes a file signature and some new 
file to compute against

```
let path = PathBuf::from("some/file/location");
let (signature, stats) = Signature::new(&path).unwrap;

let some_new_path = PathBuf::from("the/file/changed/maybe");
let (delta, stats) = Delta::new(&mut signature, &some_new_path);
```

#### Applying a delta to a file 

```
let path = PathBuf::from("some/file/location");
let (signature, stats) = Signature::new(&path).unwrap;

let some_new_path = PathBuf::from("the/file/changed/maybe");
let (delta, stats) = Delta::new(&mut signature, &some_new_path);

// will output the value of the new file as a Vec<u8> 
let new_file = patch_file(&path, delta);
```


