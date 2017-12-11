use std::io::Write;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Error;


pub use ::parser::plugin::Plugin;
pub use ::parser::records::MUSC;

fn write_ident(writer: &mut Write, ident: &str) {
    writer.write(ident.as_bytes()).unwrap();
}

fn write_zstring(writer: &mut Write, xxxx: bool, zstring: &str) {
    let bytes = zstring.as_bytes();
    if xxxx {
        writer.write_u16::<LittleEndian>(0).unwrap();
    } else {
        writer.write_u16::<LittleEndian>(bytes.len() as u16 + 1).unwrap();
    }
    writer.write(bytes).unwrap();
    writer.write_u8(0).unwrap(); // Null terminated.
}

fn write_u8(writer: &mut Write, v: u8) {
    writer.write_u8(v).unwrap();
}

fn write_u16(writer: &mut Write, v: u16) {
    writer.write_u16::<LittleEndian>(v).unwrap();
}

fn write_i32(writer: &mut Write, v: i32) {
    writer.write_i32::<LittleEndian>(v).unwrap();
}

fn write_u32(writer: &mut Write, v: u32) {
    writer.write_u32::<LittleEndian>(v).unwrap();
}

fn write_f32(writer: &mut Write, v: f32) {
    writer.write_f32::<LittleEndian>(v).unwrap();
}

fn write_u64(writer: &mut Write, v: u64) {
    writer.write_u64::<LittleEndian>(v).unwrap();
}

pub fn write_plugin(mut writer: &mut Write, plugin: &Plugin) -> Result<(), Error> {
    write_ident(&mut writer, "TES4");
    // size of the TES4 header // TODO: XXXX support
    write_u16(&mut writer,
        // "HEDR" + len + fields
        18
        // "CNAM" + len + str len + null
        + 7 + plugin.author.as_bytes().len() as u16
        // "SNAM" + len + str len + null
        + 7 + plugin.description.as_bytes().len() as u16
        // each "MAST" + len + str len + null  + "DATA" + len + u64
        + plugin.masters.iter().fold(0, |i, master| i + master.as_bytes().len() as u16 + 21)
        // "ONAM" + len + each u32
        /*+ 6 + plugin.overrides.len() as u16 * 4*/
         );
    // No idea what this is about, as there's no documentation, but every esp file has it.
    let buf: [u8; 18] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00, 0x00];
    let _ = writer.write(&buf);
    write_ident(&mut writer, "HEDR");
    write_u16(&mut writer, 12);
    write_f32(&mut writer, plugin.version);
    write_i32(&mut writer, plugin.num_records);
    write_u32(&mut writer, plugin.next_object_id);
    write_ident(&mut writer, "CNAM");
    write_zstring(&mut writer, false, plugin.author.as_str());
    write_ident(&mut writer, "SNAM");
    write_zstring(&mut writer, false, plugin.description.as_str());
    for master in plugin.masters.iter() {
        write_ident(&mut writer, "MAST");
        write_zstring(&mut writer, false, master);
        write_ident(&mut writer, "DATA");
        write_u16(&mut writer, 8); // Size of data field.
        write_u64(&mut writer, 0); // Not even used.
    }
    if plugin.overrides.len() > 0 {
        println!("Overrides? Ugh");
        let onam_bytes: u32 = plugin.overrides.len() as u32 * 4;
        let xxxx_onam = plugin.overrides.len() as u32 > onam_bytes;
        if xxxx_onam {
            write_ident(&mut writer, "XXXX");
            write_u32(&mut writer, onam_bytes);
        }
        write_ident(&mut writer, "ONAM");
        if !xxxx_onam {
            write_u16(&mut writer, onam_bytes as u16);
        } else {
            write_u16(&mut writer, 0);
        }
        for form_id in plugin.overrides.iter() {
            write_u32(&mut writer, *form_id);
        }
    }
    if plugin.music.len() > 0 {
        //println!("Writing Music");
        write_ident(&mut writer, "GRUP");
        // get size of all MUSC values contained
        let data_size: u32 = plugin.music.iter().fold(0, |s, musc| s + 67 + musc.editor_id.as_bytes().len() as u32 + musc.track_ids.len() as u32 * 4);
        //println!("GRUP size: {}", data_size + 24);
        write_u32(&mut writer, data_size + 24); // size (including header size)
        write_ident(&mut writer, "MUSC"); // label
        write_i32(&mut writer, 0); // group type. top level, so 0
        write_u16(&mut writer, 0); // stamp. we don't care about this
        write_u16(&mut writer, 0); // unknown
        write_u16(&mut writer, 0); // version. doesn't matter.
        write_u16(&mut writer, 0); // unknown
        for musc in plugin.music.iter() {
            write_ident(&mut writer, "MUSC");
            write_u32(&mut writer, 43 + musc.editor_id.as_bytes().len() as u32 + musc.track_ids.len() as u32 * 4);
            write_u32(&mut writer, 0x00000000); // flags
            write_u32(&mut writer, musc.form_id); // form id. Hope to god I don't have to do this the hard way
            write_u32(&mut writer, 0); // revision
            write_u16(&mut writer, 0); // version
            write_u16(&mut writer, 0); // unknown
            

            write_ident(&mut writer, "EDID");
            write_zstring(&mut writer, false, musc.editor_id.as_str());
            write_ident(&mut writer, "FNAM");
            write_u16(&mut writer, 4); // size of the FNAM value.
            write_u32(&mut writer, musc.flags);
            write_ident(&mut writer, "PNAM");
            write_u16(&mut writer, 4); // two u16s
            write_u16(&mut writer, musc.priority);
            write_u16(&mut writer, musc.ducking);
            write_ident(&mut writer, "WNAM");
            write_u16(&mut writer, 4);
            write_f32(&mut writer, musc.fade_duration);
            write_ident(&mut writer, "TNAM");
            write_u16(&mut writer, musc.track_ids.len() as u16 * 4);
            for track_id in musc.track_ids.iter() {
                write_u32(&mut writer, *track_id);
            }
        }
    }
    Ok(())
}