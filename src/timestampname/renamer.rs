use std::cmp::Ordering;
use std::path::PathBuf;

use super::FileMetadata;
use super::RenameOperation;
use super::failures::Failure;

fn compare_metadatas(md1: &FileMetadata, md2: &FileMetadata) -> Ordering {
    if md1.file_name == md2.file_name {
        panic!("file encountered twice: {}", md1.file_name);
    }
    let ct1 = &md1.creation_timestamp;
    let ct2 = &md2.creation_timestamp;
    match ct1 {
        ct if ct < ct2 => Ordering::Less,
        ct if ct > ct2 => Ordering::Greater,
        ct if ct == ct2 => {
            // workaround for Android way of dealing with same-second shots:
            // 20180430_184327.jpg
            // 20180430_184327(0).jpg
            let l1: usize = md1.file_name.len();
            let l2: usize = md2.file_name.len();
            match l1 {
                l if l < l2 => Ordering::Less,
                l if l > l2 => Ordering::Greater,
                l if l == l2 => {
                    // equal names case was checked at the beginning:
                    if md1.file_name < md2.file_name {
                        return Ordering::Less;
                    } else {
                        return Ordering::Greater;
                    }
                }
                _ => panic!("no match for input file length: {}, {}", l1, l2)
            }
        }
        _ => panic!("no match for input creation timestamp: {}, {}", ct1, ct2)
    }
}

fn prefix_width(count: usize) -> usize {
    match count {
        c if c < 10 => 1,
        c if c < 100 => 2,
        c if c < 1000 => 3,
        c if c < 10_000 => 4,
        c if c < 100_000 => 5,
        _ => panic!("too many files!")
    }
}

pub fn prepare_rename_operations(mut items: Vec<FileMetadata>, no_prefix: bool) -> Result<Vec<RenameOperation>, Failure> {
    let prefix_width: usize = prefix_width(items.len());
    items.sort_unstable_by(compare_metadatas);
    let sorted: Vec<FileMetadata> = items;

    let mut operations: Vec<RenameOperation> = Vec::new();
    for (i, f) in sorted.iter().enumerate() {
        let ext: String = PathBuf::from(&f.file_name)
            .extension()
            .and_then(|x| x.to_str())
            .map_or("".to_string(), |x| format!(".{}", x));
        let to: String = match no_prefix {
            true => format!("{}{}",
                            f.creation_timestamp,
                            ext),
            false => format!("{:width$}-{}{}",
                             i + 1,
                             f.creation_timestamp,
                             ext,
                             width = prefix_width)
        };
        let operation = RenameOperation {
            from: f.file_name.to_string(),
            to,
        };
        operations.push(operation);
    }

    return Ok(operations);
}
