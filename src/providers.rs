use windows::{
    core::{Result, GUID},
    Win32::{
        Foundation::{ERROR_INSUFFICIENT_BUFFER, WIN32_ERROR},
        System::Diagnostics::Etw::{TdhEnumerateProviders, PROVIDER_ENUMERATION_INFO},
    },
};

pub struct ProviderInfo {
    pub name: String,
    pub guid: GUID,
    pub schema_source: u32,
}

pub fn enumerate_providers() -> Result<Vec<ProviderInfo>> {
    // Determine the size of the provider list
    let mut buffer_size = unsafe {
        let mut buffer_size = 0;
        let result = TdhEnumerateProviders(None, &mut buffer_size);
        if result != ERROR_INSUFFICIENT_BUFFER.0 {
            WIN32_ERROR(result).ok()?;
        }
        buffer_size
    };

    let providers = if buffer_size > 0 {
        unsafe {
            let mut buffer = vec![0u8; buffer_size as usize];
            let providers_ptr = buffer.as_mut_ptr() as *mut PROVIDER_ENUMERATION_INFO;
            let result = TdhEnumerateProviders(Some(providers_ptr), &mut buffer_size);
            WIN32_ERROR(result).ok()?;

            let providers_enum_info = providers_ptr.as_ref().unwrap();
            let num_providers = providers_enum_info.NumberOfProviders as usize;
            let providers = if num_providers > 0 {
                let providers_ptr = providers_enum_info.TraceProviderInfoArray.as_ptr();
                let mut providers = Vec::with_capacity(num_providers);
                for i in 0..num_providers {
                    let provider_ptr = providers_ptr.add(i);
                    let provider = provider_ptr.as_ref().unwrap();
                    let name_offset = provider.ProviderNameOffset;
                    let name_ptr = buffer[name_offset as usize..].as_ptr() as *const u16;

                    let name_length = {
                        let mut current_char = name_ptr;
                        let mut length = 0;
                        while *current_char != 0 {
                            current_char = current_char.add(1);
                            length += 1;
                        }
                        length
                    };

                    let name_raw_slice = std::slice::from_raw_parts(name_ptr, name_length);
                    let name = String::from_utf16(name_raw_slice).unwrap();
                    providers.push(ProviderInfo {
                        name,
                        guid: provider.ProviderGuid,
                        schema_source: provider.SchemaSource,
                    });
                }
                providers
            } else {
                Vec::new()
            };
            providers
        }
    } else {
        Vec::new()
    };
    Ok(providers)
}
