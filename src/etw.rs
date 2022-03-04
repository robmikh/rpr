use bytemuck::offset_of;
use windows::{
    core::{Result, GUID},
    Win32::{
        Foundation::{ERROR_WMI_INSTANCE_NOT_FOUND, WIN32_ERROR},
        System::Diagnostics::Etw::{
            ControlTraceW, EnableTraceEx2, StartTraceW, EVENT_CONTROL_CODE_ENABLE_PROVIDER,
            EVENT_TRACE_CONTROL_QUERY, EVENT_TRACE_CONTROL_STOP, EVENT_TRACE_FILE_MODE_SEQUENTIAL,
            EVENT_TRACE_PROPERTIES, TRACE_LEVEL_VERBOSE, WNODE_FLAG_TRACED_GUID,
        },
    },
};

use crate::result::ToWindowsResult;

const INFINITE: u32 = 0xFFFFFFFF;

#[repr(C)]
pub struct EventTraceProperties {
    pub properties: EVENT_TRACE_PROPERTIES,
    logger_name: [u16; 256],
    log_file_name: [u16; 256],
}

impl EventTraceProperties {
    pub fn new(session_name: &str) -> Self {
        let wide_logger_name: Vec<u16> = session_name.encode_utf16().collect();
        let wide_log_file_name: Vec<u16> = format!("{}.etl", session_name).encode_utf16().collect();
        let mut logger_name_array = [0u16; 256];
        let mut log_file_name_array = [0u16; 256];
        logger_name_array[0..wide_logger_name.len()].copy_from_slice(&wide_logger_name);
        log_file_name_array[0..wide_log_file_name.len()].copy_from_slice(&wide_log_file_name);
        let mut result = Self {
            properties: Default::default(),
            logger_name: logger_name_array,
            log_file_name: log_file_name_array,
        };
        result.properties.Wnode.BufferSize = std::mem::size_of::<EventTraceProperties>() as u32;
        result.properties.LoggerNameOffset =
            offset_of!(result, EventTraceProperties, logger_name) as u32;
        result.properties.LogFileNameOffset =
            offset_of!(result, EventTraceProperties, log_file_name) as u32;
        result
    }
}

pub fn start_trace(session_name: &str, provider_id: &GUID) -> Result<u64> {
    let mut properties = EventTraceProperties::new(session_name);
    properties.properties.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    properties.properties.BufferSize = 1024;
    properties.properties.LogFileMode = EVENT_TRACE_FILE_MODE_SEQUENTIAL;
    properties.properties.MinimumBuffers = 300;
    properties.properties.FlushTimer = 1;

    let mut handle = 0;
    unsafe {
        WIN32_ERROR(StartTraceW(
            &mut handle,
            session_name,
            &mut properties.properties,
        ))
        .ok()?
    };
    assert_ne!(handle, 0);
    unsafe {
        WIN32_ERROR(EnableTraceEx2(
            handle,
            provider_id,
            EVENT_CONTROL_CODE_ENABLE_PROVIDER.0,
            TRACE_LEVEL_VERBOSE as u8,
            0,
            0,
            INFINITE,
            std::ptr::null(),
        ))
        .ok()?;
    };
    Ok(handle)
}

pub fn stop_trace(session_name: &str) -> Result<()> {
    let mut properties = EventTraceProperties::new(session_name);
    let error = unsafe {
        WIN32_ERROR(ControlTraceW(
            0,
            session_name,
            &mut properties.properties,
            EVENT_TRACE_CONTROL_QUERY,
        ))
    };
    if error == ERROR_WMI_INSTANCE_NOT_FOUND {
        return Ok(());
    }
    error.ok()?;
    let handle = unsafe { properties.properties.Wnode.Anonymous1.HistoricalContext };
    assert_ne!(handle, 0);
    unsafe {
        WIN32_ERROR(ControlTraceW(
            handle,
            None,
            &mut properties.properties,
            EVENT_TRACE_CONTROL_STOP,
        ))
        .ok()?;
    };
    Ok(())
}
