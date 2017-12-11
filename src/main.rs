#![allow(dead_code)]
#![allow(non_camel_case_types)]
extern crate winreg;
extern crate byteorder;

use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use std::env;

use std::io::BufReader;
use std::io::BufRead;
use winreg::RegKey;

mod parser;
use parser::Parser;
use parser::Plugin;

// TODO: Refactor EVERYTHING.

#[allow(dead_code)]
const DEFAULT_S_INSTALL_DIR: &'static str = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\skyrim";
#[allow(dead_code)]
const DEFAULT_SSE_INSTALL_DIR: &'static str = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Skyrim Special Edition";

#[allow(dead_code)]
const S_REGISTRY_KEY_X32: &'static str = "SOFTWARE\\Bethesda Softworks\\skyrim"; // registry key location for Skyrim on a 32 bit Windows OS.
#[allow(dead_code)]
const S_REGISTRY_KEY_X64: &'static str = "SOFTWARE\\WOW6432NODE\\Bethesda Softworks\\skyrim"; // registry key location for Skyrim on a 64 bit Windows OS.

const SSE_REGISTRY_KEY: &'static str = "SOFTWARE\\WOW6432NODE\\Bethesda Softworks\\Skyrim Special Edition"; // Skyrim Special Edition registry key location.

fn get_sse_install() -> String {
  let hklm = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
  let sse_key = hklm.open_subkey_with_flags(SSE_REGISTRY_KEY, winreg::enums::KEY_READ).unwrap();
  sse_key.get_value("installed path").expect("Unable to retrieve Skyrim Special Edition install location.")
}

fn get_sse_load_order() -> Vec<String> {
  let mut path_buf = env::home_dir().expect("Unable to retrieve home directory.");
  path_buf.push("AppData");
  path_buf.push("Local");
  path_buf.push("Skyrim Special Edition");
  path_buf.push("plugins");
  path_buf.set_extension("txt");
  let path = path_buf.as_path();
  let plugins_file = File::open(path).expect("Unable to read plugins.txt.");
  let buf_reader = BufReader::new(plugins_file);
  let mut entries: Vec<String> = vec![String::from("Skyrim.esm"), String::from("Update.esm"), String::from("Dawnguard.esm"), String::from("Hearthfires.esm"), String::from("Dragonborn.esm")];
  for line in buf_reader.lines() {
    let s = line.unwrap();
    if s.starts_with('*') {
      entries.push(String::from(&s[1 ..]));
    }
  }
  entries
}

fn handle_plugin(p: &Plugin) -> HashMap<String, Vec<u32>> {
  //println!("Analyzing {}", p.path().to_str().unwrap());
  let mut map: HashMap<String, Vec<u32>> = HashMap::new();

  for musc in p.music.iter() {
    map.insert(musc.editor_id.clone(), musc.track_ids.clone());
  }

  //println!("\tMUSIC: {:?}", p.music);
  if p.music.len() > 0 {
    println!("\tFound {} MUSC records", p.music.len());
  }

  map
}

fn main() {
  let output_name = "music_merge_patch.esp";
  let mut output_masters: Vec<String> = vec!();

  let install_dir = get_sse_install();
  println!("Skyrim Special Edition installed to:\n\t{}", install_dir);
  let load_order = get_sse_load_order();
  let install_path = Path::new(install_dir.as_ref() as &str).join(Path::new("Data"));
  let plugin_parser = Parser::new();
  let mut music_map: HashMap<String, Vec<u32>> = HashMap::new();

  for plugin_entry in load_order.iter() {
    let plugin_path = install_path.join(Path::new(plugin_entry));
    let plugin_file_name = plugin_path.file_name().unwrap().to_str().unwrap();
    if plugin_file_name != output_name {
      println!("{}", plugin_file_name);
      if !plugin_path.exists() {
        println!("Unable to find {}", plugin_path.to_str().unwrap());
      } else {
        let result = plugin_parser.parse(plugin_path.as_path());
        match result {
          Ok(plugin) => {
            let plugin_music = handle_plugin(&plugin);
            if plugin_music.len() > 0 {
                for master in plugin.masters {
                  if !output_masters.contains(&master) {
                    output_masters.push(master.clone());
                  }
                  output_masters.push(String::from(plugin_file_name));
                }
              for (editor_id, track_ids) in plugin_music.iter() {
                let vec = music_map.entry(editor_id.clone()).or_insert(vec!());
                for track_id in track_ids.iter() {
                  if !vec.contains(track_id) {
                    vec.push(*track_id);
                  }
                }
              }
            }
          },
          Err(err) => { println!("[Error] {}", err); break;}
        }
      }
    }
  }
  let mut output_plugin = Plugin::new(&install_path.join(Path::new(output_name)));
  output_plugin.author = String::from("ESMusicMerger");
  output_plugin.description = String::from(format!("Collection of music from {:?}", output_plugin.masters));
  output_plugin.masters = output_masters.clone();

  println!("{:?}", output_masters);
  //println!("{:?}", music_map);
}
