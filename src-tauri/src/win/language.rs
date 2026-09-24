use windows::Win32::Globalization::{
    GetUserPreferredUILanguages,
    MUI_LANGUAGE_NAME,
};
use windows::core::PWSTR;

pub fn get_windows_language() -> Result<String, String> {

    let mut language_count = 0u32;
    let mut buffer_length = 0u32;

    unsafe {
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut language_count,
            None,
            &mut buffer_length,
        )
    }
    .map_err(|e| e.to_string())?;

    let mut buffer = vec![0u16; buffer_length as usize];

    unsafe {
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut language_count,
            Some(PWSTR(buffer.as_mut_ptr())),
            &mut buffer_length,
        )
    }
    .map_err(|e| e.to_string())?;

    let languages = buffer
        .split(|&c| c == 0)
        .filter(|part| !part.is_empty())
        .filter_map(|part| String::from_utf16(part).ok())
        .collect::<Vec<_>>();
    // return Ok(String::from("ru-RU"));
    languages
        .into_iter()
        .next()
        .ok_or_else(|| "No Windows UI language found".into())
}