//! The infix for rotated files contains an index number.
use super::{InfixFilter, CURRENT_INFIX};
use crate::{writers::FileLogWriterConfig, FileSpec};
use std::cmp::max;

pub(super) fn number_infix(idx: u32) -> String {
    format!("r{idx:0>5}")
}

pub(super) fn index_for_rcurrent(
    config: &FileLogWriterConfig,
    o_index_for_rcurrent: Option<u32>,
    rotate_rcurrent: bool,
) -> Result<u32, std::io::Error> {
    // we believe what we get - but if we get nothing, we determine what's next
    // according to the filesystem
    let mut index_for_rcurrent = o_index_for_rcurrent
        .or_else(|| get_highest_index(&config.file_spec).map(|idx| idx + 1))
        .unwrap_or(0);

    if rotate_rcurrent {
        match std::fs::rename(
            config.file_spec.as_pathbuf(Some(CURRENT_INFIX)),
            config
                .file_spec
                .as_pathbuf(Some(&number_infix(index_for_rcurrent))),
        ) {
            Ok(()) => {
                index_for_rcurrent += 1;
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(e);
                }
            }
        }
    }
    Ok(index_for_rcurrent)
}

pub(super) fn get_highest_index(file_spec: &FileSpec) -> Option<u32> {
    let mut o_highest_idx = None;
    for file in
        super::list_and_cleanup::list_of_log_and_compressed_files(file_spec, &InfixFilter::Numbrs)
    {
        let name = file.file_stem().unwrap(/*ok*/).to_string_lossy();
        let infix = if file_spec.has_basename()
            || file_spec.has_discriminant()
            || file_spec.uses_timestamp()
        {
            // infix is the last, but not the first part of the name, starts with _r
            match name.rsplit("_r").next() {
                Some(infix) => infix,
                None => continue, // ignore unexpected files
            }
        } else {
            // infix is the only part of the name, just skip over the r
            &name[1..]
        };

        let idx: u32 = infix.parse().unwrap_or(0);
        o_highest_idx = match o_highest_idx {
            None => Some(idx),
            Some(prev) => Some(max(prev, idx)),
        };
    }
    o_highest_idx
}
