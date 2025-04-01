use bytemuck::offset_of;
use windows::{
    Win32::{
        Foundation::ERROR_WMI_INSTANCE_NOT_FOUND,
        System::Diagnostics::Etw::{
            CONTROLTRACE_HANDLE, ControlTraceW, EVENT_CONTROL_CODE_ENABLE_PROVIDER,
            EVENT_TRACE_CONTROL_QUERY, EVENT_TRACE_CONTROL_STOP, EVENT_TRACE_FILE_MODE_SEQUENTIAL,
            EVENT_TRACE_PROPERTIES, EnableTraceEx2, StartTraceW, TRACE_LEVEL_VERBOSE,
            WNODE_FLAG_TRACED_GUID,
        },
    },
    core::{GUID, HSTRING, Result},
};

const INFINITE: u32 = 0xFFFFFFFF;

#[repr(C)]
pub struct EventTraceProperties {
    pub properties: EVENT_TRACE_PROPERTIES,
    logger_name: [u16; 256],
    log_file_name: [u16; 256],
}

impl EventTraceProperties {
    fn basic(session_name: &str, file: &str) -> Self {
        let wide_logger_name: Vec<u16> = session_name.encode_utf16().collect();
        let wide_log_file_name: Vec<u16> = file.encode_utf16().collect();
        let mut result = Self {
            properties: Default::default(),
            logger_name: [0u16; 256],
            log_file_name: [0u16; 256],
        };
        result.properties.Wnode.BufferSize = std::mem::size_of::<EventTraceProperties>() as u32;
        result.properties.LoggerNameOffset =
            offset_of!(result, EventTraceProperties, logger_name) as u32;
        result.properties.LogFileNameOffset =
            offset_of!(result, EventTraceProperties, log_file_name) as u32;
        result.logger_name[0..wide_logger_name.len()].copy_from_slice(&wide_logger_name);
        result.log_file_name[0..wide_log_file_name.len()].copy_from_slice(&wide_log_file_name);
        result
    }

    pub fn new(session_name: &str) -> Self {
        Self::basic(session_name, "")
    }

    pub fn new_with_file(session_name: &str, file: &str) -> Self {
        Self::basic(session_name, file)
    }
}

pub fn start_trace(
    session_name: &str,
    file: &str,
    provider_id: &GUID,
) -> Result<CONTROLTRACE_HANDLE> {
    let mut properties = EventTraceProperties::new_with_file(session_name, file);
    properties.properties.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    properties.properties.BufferSize = 1024;
    properties.properties.LogFileMode = EVENT_TRACE_FILE_MODE_SEQUENTIAL;
    properties.properties.MinimumBuffers = 300;
    properties.properties.FlushTimer = 1;

    let mut handle = CONTROLTRACE_HANDLE::default();
    unsafe {
        StartTraceW(
            &mut handle,
            &HSTRING::from(session_name),
            &mut properties.properties,
        )
        .ok()?
    };
    assert_ne!(handle.Value, 0);
    unsafe {
        EnableTraceEx2(
            handle,
            provider_id,
            EVENT_CONTROL_CODE_ENABLE_PROVIDER.0,
            TRACE_LEVEL_VERBOSE as u8,
            0,
            0,
            INFINITE,
            None,
        )
        .ok()?;
    };
    Ok(handle)
}

pub fn stop_trace(session_name: &str) -> Result<bool> {
    let mut properties = EventTraceProperties::new(session_name);
    let error = unsafe {
        ControlTraceW(
            CONTROLTRACE_HANDLE { Value: 0 },
            &HSTRING::from(session_name),
            &mut properties.properties,
            EVENT_TRACE_CONTROL_QUERY,
        )
    };
    if error == ERROR_WMI_INSTANCE_NOT_FOUND {
        return Ok(false);
    }
    error.ok()?;
    let handle = unsafe { properties.properties.Wnode.Anonymous1.HistoricalContext };
    assert_ne!(handle, 0);
    unsafe {
        ControlTraceW(
            CONTROLTRACE_HANDLE { Value: handle },
            None,
            &mut properties.properties,
            EVENT_TRACE_CONTROL_STOP,
        )
        .ok()?;
    };
    Ok(true)
}
