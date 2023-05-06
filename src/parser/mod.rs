use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::path::Path;
use std::str::from_utf8;

mod plugin;
pub use plugin::*;
pub mod records;
use records::*;
pub mod plugin_writer;

/// Read zstrings given a length
fn read_zstring(reader: &mut dyn Read, len: u32) -> String {
    let mut buf: Vec<u8> = vec![];
    if len > 0 {
        // `(len - 1)` then skip the next byte because we want to exlude the \0 from our string
        let _ = reader.take((len - 1) as u64).read_to_end(&mut buf);
        let _ = reader.read(&mut [0; 1]); // Skip the \0
    } else {
        loop {
            let c = reader.read_u8().unwrap();
            if c == 0 {
                break;
            } else {
                buf.push(c);
            }
        }
    }
    String::from_utf8(buf).expect("Invalid UTF8")
}

/// Read record identifiers
fn read_ident(reader: &mut dyn Read) -> String {
    let mut buf: [u8; 4] = [0; 4];
    let _ = reader.read_exact(&mut buf).unwrap();
    //println!("{:?}", buf);
    String::from(from_utf8(&buf).unwrap())
}

fn read_u8(reader: &mut dyn Read) -> u8 {
    reader.read_u8().unwrap()
}
fn read_u16(reader: &mut dyn Read) -> u16 {
    reader.read_u16::<LittleEndian>().unwrap()
}
fn read_i32(reader: &mut dyn Read) -> i32 {
    reader.read_i32::<LittleEndian>().unwrap()
}
fn read_u32(reader: &mut dyn Read) -> u32 {
    reader.read_u32::<LittleEndian>().unwrap()
}
fn read_f32(reader: &mut dyn Read) -> f32 {
    reader.read_f32::<LittleEndian>().unwrap()
}
fn read_u64(reader: &mut dyn Read) -> u64 {
    reader.read_u64::<LittleEndian>().unwrap()
}
fn skip(reader: &mut dyn Read, len: u64) {
    /*reader.fill_buf();
    reader.consume(len);*/
    //reader.seek(SeekFrom::Current(len as i64));
    let mut _trash = vec![];
    reader.take(len).read_to_end(&mut _trash).unwrap();
    //println!("\tSkipped {} bytes", reader.take(len).read_to_end(&mut _trash).unwrap());
}

fn parse_record_header(mut reader: &mut dyn Read) -> RecordHeader {
    RecordHeader {
        record_type: read_ident(&mut reader),
        size: read_u32(&mut reader),
        flags: read_u32(&mut reader),
        id: read_u32(&mut reader),
        revision: read_u32(&mut reader),
        version: read_u16(&mut reader),
        unknown: read_u16(&mut reader),
    }
}

pub fn parse(p: &Path) -> Result<Plugin, Error> {
    println!("+Parsing `{}`", p.file_name().unwrap().to_str().unwrap());
    let mut plugin = Plugin::new(p);
    let file: File = File::open(p).expect("Unable to open file!");
    let mut reader = BufReader::with_capacity(256, file);

    // Magic bytes
    if read_ident(&mut reader) != "TES4" {
        return Result::Err(Error::new(
            ErrorKind::InvalidData,
            String::from("Not a valid ESM/ESP file"),
        ));
    }
    // skip the random junk between TES4 and HEDR
    skip(&mut reader, 20);

    // Extended field size
    let mut xxxx = false;
    let mut x_len: u32 = 0;

    // Header
    let mut ident = read_ident(&mut reader);
    assert!(ident.as_str() == "HEDR");
    let hedr_size = read_u16(&mut reader);
    if hedr_size != 12 {
        println!(
            "HEDR has unusual size field ({}). Ignoring. If you get weird errors,\
            please verify that the file is not corrupt and submit a bug report with \
            your file.",
            hedr_size
        );
    }
    plugin.version = read_f32(&mut reader);
    plugin.num_records = read_i32(&mut reader);
    plugin.next_object_id = read_u32(&mut reader);
    if plugin.version != 1.7 && plugin.version != 0.94 {
        panic!("{}", format!("Incorrect version!: {}", plugin.version));
    }
    ident = read_ident(&mut reader);
    loop {
        let mut len: u32 = read_u16(&mut reader) as u32;
        if xxxx {
            len = x_len;
            //println!("\tXXXX: {}", x_len);
            xxxx = false;
        }
        match ident.as_str() {
            "MAST" => {
                let name = read_zstring(&mut reader, len);
                //println!("\tMAST: `{}`", &name);
                plugin.masters.push(name);
                skip(&mut reader, 14);
            }
            "XXXX" => {
                xxxx = true;
                x_len = read_u32(&mut reader);
            }
            "CNAM" => plugin.author = read_zstring(&mut reader, len),
            "SNAM" => plugin.description = read_zstring(&mut reader, len),
            "INTV" => plugin.intv = read_u32(&mut reader),
            "ONAM" => {
                //println!("\tOverrides len: {}", len);
                for _ in 0..len / 4 {
                    plugin.overrides.push(read_u32(&mut reader));
                }
                //skip(&mut reader, len as u64); // NYI
            }
            "INCC" => {
                // Unknown
                read_u32(&mut reader);
            }
            _ => {
                // Invalid data. Time to panic.
                panic!("{}", format!("\tUnknown: `{}`", ident));
            }
        }
        // Nothing left to read. A plugin can
        // be just the header, which will just
        // cause the .bsa file for it to load.
        if reader.fill_buf().unwrap().len() == 0 {
            break;
        }
        ident = read_ident(&mut reader);
        if ident.as_str() == "GRUP" {
            // End of the header. Moving on.
            break;
        }
    }

    //println!("\tFinished parsing header");
    while ident.as_str() == "GRUP" {
        // Group length includes header size: 24 bytes
        let group_len = read_u32(&mut reader) - 24;
        let mut label = [0; 4];
        let _ = reader.read_exact(&mut label).unwrap();
        let label_str;
        let group_type = read_i32(&mut reader);
        let _stamp = read_u16(&mut reader);
        let _unknown = read_u16(&mut reader);
        let _version = read_u16(&mut reader);
        let _unknown2 = read_u16(&mut reader);
        if group_type != 0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Expected top level group",
            ));
        }
        label_str = from_utf8(&label).unwrap();
        //println!("Found GRUP<{}>[{}]", group_type, group_len);
        if label_str == "MUSC" {
            let mut pos = 0;
            while pos < group_len {
                let record_header = parse_record_header(&mut reader);
                if read_ident(&mut reader) != "EDID" {
                    return Err(Error::new(ErrorKind::InvalidData, "Expected EDID"));
                }
                let edid_len = read_u16(&mut reader) as u32;
                let editor_id = read_zstring(&mut reader, edid_len);
                if read_ident(&mut reader) != "FNAM" {
                    return Err(Error::new(ErrorKind::InvalidData, "Expected FNAM"));
                }
                read_u16(&mut reader);
                let flags = read_u32(&mut reader);
                if read_ident(&mut reader) != "PNAM" {
                    return Err(Error::new(ErrorKind::InvalidData, "Expected PNAM"));
                }
                read_u16(&mut reader);
                let priority = read_u16(&mut reader);
                let ducking = read_u16(&mut reader);
                if read_ident(&mut reader) != "WNAM" {
                    return Err(Error::new(ErrorKind::InvalidData, "Expected WNAM"));
                }
                read_u16(&mut reader);
                let fade_duration = read_f32(&mut reader);
                if read_ident(&mut reader) != "TNAM" {
                    return Err(Error::new(ErrorKind::InvalidData, "Expected TNAM"));
                }
                let data_len = read_u16(&mut reader) as u32;
                let mut track_ids: Vec<u32> = vec![];
                for _ in 0..data_len / 4 {
                    track_ids.push(read_u32(&mut reader));
                }
                pos += 67 + editor_id.as_bytes().len() as u32 + track_ids.len() as u32 * 4;
                plugin.music.push(MUSC {
                    form_id: record_header.id,
                    editor_id: editor_id,
                    flags: flags,
                    priority: priority,
                    ducking: ducking,
                    fade_duration: fade_duration,
                    track_ids: track_ids,
                });
            }
        } else {
            // We don't care about whatever this is.
            skip(&mut reader, group_len as u64);
        }
        // EOF
        if reader.fill_buf().unwrap().len() == 0 {
            break;
        }
        ident = read_ident(&mut reader);
    }
    //println!("\tAuthor: {}\n\tDescription: {}", plugin.author, plugin.description);
    /*for musc in plugin.music.iter() {
        println!("\tMUSC {}[{}]", musc.editor_id, musc.track_ids.len());
        println!("{:?}", musc.track_ids);
    }*/
    Ok(plugin)
}
