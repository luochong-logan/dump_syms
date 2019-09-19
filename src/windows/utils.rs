// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::path::PathBuf;
use symbolic_debuginfo::pe::PeObject;

use crate::utils;

pub fn get_win_path(path: &str) -> PathBuf {
    PathBuf::from(path.replace("\\", "/"))
}

fn try_to_find_pdb(path: PathBuf, pdb_filename: &str) -> Option<Vec<u8>> {
    // Just check that the file is in the same directory as the PE one
    let pdb = path.with_file_name(pdb_filename);
    if pdb.is_file() {
        Some(utils::read_file(pdb))
    } else {
        // We try in CWD
        let mut pdb = std::env::current_dir().expect("Unable to get the current working directory");
        pdb.set_file_name(pdb_filename);
        if pdb.is_file() {
            Some(utils::read_file(pdb))
        } else {
            None
        }
    }
}

#[cfg(windows)]
fn os_specific_try_to_find_pdb(path: PathBuf, pdb_filename: String) -> (Option<Vec<u8>>, String) {
    // We've probably a win path: C:\foo\bar\toto.pdb and we're on a windows machine
    // so we can try to check if this file exists.
    let pdb_path = PathBuf::from(pdb_filename);
    let pdb_name = pdb_path.file_name().unwrap().to_str().unwrap().to_string();

    if pdb_path.is_file() {
        (Some(utils::read_file(pdb_path)), pdb_name)
    } else {
        (try_to_find_pdb(path, &pdb_name), pdb_name)
    }
}

#[cfg(unix)]
fn os_specific_try_to_find_pdb(path: PathBuf, pdb_filename: String) -> (Option<Vec<u8>>, String) {
    // We've probably a win path: C:\foo\bar\toto.pdb and we're on a unix machine
    // so no need to look for this path.
    // Just change the \ to / to be able to call file_name()
    // (else it won't work since "C:\foo\bar\toto.pdb" is a correct filename)
    let pdb_path = get_win_path(&pdb_filename);
    let pdb_name = pdb_path.file_name().unwrap().to_str().unwrap().to_string();
    (try_to_find_pdb(path, &pdb_name), pdb_name)
}

pub fn get_pe_pdb_buf<'a>(path: PathBuf, buf: &'a [u8]) -> Option<(PeObject<'a>, Vec<u8>, String)> {
    let pe = PeObject::parse(&buf)
        .unwrap_or_else(|_| panic!("Unable to parse the PE file {}", path.to_str().unwrap()));
    if let Some(pdb_filename) = pe.debug_file_name() {
        let pdb_filename = pdb_filename.into_owned();
        let (pdb, pdb_name) = os_specific_try_to_find_pdb(path, pdb_filename);

        if let Some(pdb_buf) = pdb {
            Some((pe, pdb_buf, pdb_name))
        } else {
            None
        }
    } else {
        None
    }
}
