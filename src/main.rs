use bytemuck::offset_of;
use clap::{Parser, Subcommand};
use windows::{
    core::{Result, GUID},
    Win32::System::Diagnostics::Etw::{
        ControlTraceW, EnableTraceEx2, StartTraceW, EVENT_CONTROL_CODE_ENABLE_PROVIDER,
        EVENT_TRACE_CONTROL_QUERY, EVENT_TRACE_CONTROL_STOP, EVENT_TRACE_FILE_MODE_SEQUENTIAL,
        EVENT_TRACE_PROPERTIES, TRACE_LEVEL_VERBOSE, WNODE_FLAG_TRACED_GUID,
    },
};

const INFINITE: u32 = 0xFFFFFFFF;

#[repr(C)]
struct EventTraceProperties {
    properties: EVENT_TRACE_PROPERTIES,
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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Start {
        #[clap(short, long)]
        name: String,
        #[clap(short, long)]
        provider: String,
    },
    Stop {
        #[clap(short, long)]
        name: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Start { name, provider } => {
            stop_trace(&name);

            let provider_id: GUID = provider.as_str().into();
            let _handle = start_trace(&name, &provider_id);
        }
        Commands::Stop { name } => {
            stop_trace(&name);
        }
    }

    Ok(())
}

fn start_trace(session_name: &str, provider_id: &GUID) -> u64 {
    let mut properties = EventTraceProperties::new(session_name);
    properties.properties.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    properties.properties.BufferSize = 1024;
    properties.properties.LogFileMode = EVENT_TRACE_FILE_MODE_SEQUENTIAL;
    properties.properties.MinimumBuffers = 300;
    properties.properties.FlushTimer = 1;

    let mut handle = 0;
    let status = unsafe { StartTraceW(&mut handle, session_name, &mut properties.properties) };
    if status != 0 {
        panic!("{}", status);
    }
    let status = unsafe {
        EnableTraceEx2(
            handle,
            provider_id,
            EVENT_CONTROL_CODE_ENABLE_PROVIDER.0,
            TRACE_LEVEL_VERBOSE as u8,
            0,
            0,
            INFINITE,
            std::ptr::null(),
        )
    };
    if status != 0 {
        panic!("{}", status);
    }
    handle
}

fn stop_trace(session_name: &str) {
    let mut properties = EventTraceProperties::new(session_name);
    let status = unsafe {
        ControlTraceW(
            0,
            session_name,
            &mut properties.properties,
            EVENT_TRACE_CONTROL_QUERY,
        )
    };
    if status != 0 {
        return;
    }
    let handle = unsafe { properties.properties.Wnode.Anonymous1.HistoricalContext };
    //println!("handle: {}", handle);
    let status = unsafe {
        ControlTraceW(
            handle,
            None,
            &mut properties.properties,
            EVENT_TRACE_CONTROL_STOP,
        )
    };
    if status != 0 {
        panic!("{}", status);
    }
}
