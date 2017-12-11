#[derive(Debug)]
#[derive(Clone)]
pub struct MUSC {
    pub form_id: u32,
    pub editor_id: String,
    pub flags: u32,
    pub priority: u16,
    pub ducking: u16,
    pub fade_duration: f32,
    pub track_ids: Vec<u32>
}

#[derive(Debug)]
#[derive(Clone)]
pub struct RecordHeader {
    pub record_type: String,
    pub size: u32,
    pub flags: u32,
    pub id: u32,
    pub revision: u32,
    pub version: u16,
    pub unknown: u16
}