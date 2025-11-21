// https://en.wikipedia.org/wiki/Tar_(computing)#UStar_format
// https://www.ibm.com/docs/en/zos/2.4.0?topic=formats-tar-format-tar-archives
// use std::fs;

use std::{
    fs::{self},
    os::unix::fs::MetadataExt,
    time::UNIX_EPOCH,
};

#[derive(Debug)]
pub enum Element {
    File(Box<FileData>),
    Dir(Vec<Element>, Box<FileData>),
}

// pub struct ReadDirError(String);

#[repr(u8)]
pub enum TypeFlag {
    T0 = 0,   // Normal file
    T1 = 1,   // Hard link
    T2 = 2,   // Sym link
    T3 = 3,   // Character special
    T4 = 4,   // Block special
    T5 = 5,   // Dir
    T6 = 6,   // FIFO (named pipe)
    T7 = 7,   // Contiguous file
    G = 0x67, // Global extended header with meta data (POSIX.1-2001)
    X = 0x78, // Extended header with metadata for the next file in the archive (POSIX.1-2001)
              // A - Z unsupported
              // Other values unused in the standard
}

#[derive(Debug)]
// #[repr([u8; 257])]
pub struct FileData {
    path: [u8; 100],
    mode: [u8; 8],
    owner_id: [u8; 8],
    group_id: [u8; 8],
    file_size: [u8; 12],
    last_mod_time: [u8; 12],
    checksum: [u8; 8],
    type_flag: u8,
    name_liked_file: [u8; 100],
}

impl Default for FileData {
    fn default() -> Self {
        Self {
            path: [0; 100],
            mode: Default::default(),
            owner_id: Default::default(),
            group_id: Default::default(),
            file_size: Default::default(),
            last_mod_time: Default::default(),
            checksum: Default::default(),
            type_flag: Default::default(),
            name_liked_file: [0; 100],
        }
    }
}

impl Element {
    #[must_use]
    pub fn get_binary_header(&self) -> [u8; 512] {
        let fd = match self {
            Element::File(file_data) | Element::Dir(_, file_data) => file_data,
        };
        let mut res = [0_u8; 512];
        res[0..100].copy_from_slice(&fd.path);
        res[100..108].copy_from_slice(&fd.mode);
        res[108..116].copy_from_slice(&fd.owner_id);
        res[116..124].copy_from_slice(&fd.group_id);
        res[124..136].copy_from_slice(&fd.file_size);
        res[136..148].copy_from_slice(&fd.last_mod_time);
        res[148..156].copy_from_slice(&fd.checksum);
        res[156] = fd.type_flag;
        res[157..257].copy_from_slice(&fd.name_liked_file);
        res
    }
}

#[inline]
#[must_use]
/// # Panics
pub fn get_elements_from_path(path: &String) -> Vec<Element> {
    let files =
        fs::read_dir(path).unwrap_or_else(|_| panic!("Error while reading directory {path}"));

    let mut elements = Vec::new();

    for dir_entry in files {
        let file =
            dir_entry.unwrap_or_else(|_| panic!("Error while reading file in directory {path}"));

        assert!(file.file_name().len() <= 100, "File name too long");

        let file_metadata = file.metadata().expect("Error while readig file metadata");

        let fname = file
            .file_name()
            .into_string()
            .expect("Error while reading file name");

        let fpath = format!("{path}/{fname}");
        let mode = file_metadata.mode();
        let uid = file_metadata.uid();
        let gid = file_metadata.gid();
        let fsize = file_metadata.size();
        let mtime = file_metadata.mtime();
        let cksum = 1_234_567_u64;
        let type_flag = TypeFlag::T0 as u8;
        let link_file_name = String::new();

        let mut fd = FileData::default();
        fd.path[..fpath.len()].copy_from_slice(fpath.as_bytes());

        fd.mode
            .copy_from_slice(format!("{:07o}{}", mode, '\0').as_bytes());

        fd.owner_id
            .copy_from_slice(format!("{:07o}{}", uid, '\0').as_bytes());

        fd.group_id
            .copy_from_slice(format!("{:07o}{}", gid, '\0').as_bytes());

        fd.file_size
            .copy_from_slice(format!("{:11o}{}", fsize, '\0').as_bytes());

        fd.last_mod_time
            .copy_from_slice(format!("{:11o}{}", mtime, '\0').as_bytes());

        fd.checksum
            .copy_from_slice(format!("{:07o}{}", cksum, '\0').as_bytes());

        fd.type_flag = type_flag;

        fd.name_liked_file[..link_file_name.len()].copy_from_slice(link_file_name.as_bytes());

        if file_metadata.is_dir() {
            elements.push(Element::Dir(vec![], Box::new(fd)));
        } else {
            elements.push(Element::File(Box::new(fd)));
        }
    }
    elements
}
