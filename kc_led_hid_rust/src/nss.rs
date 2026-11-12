use crate::{Direction, Effect, HidError, KEY_COUNT, KeyLayout, Profile, Result, Rgb};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
};

const TIMER_SECONDS: [u32; 14] = [
    0, 5, 10, 15, 30, 45, 60, 120, 300, 600, 1800, 3600, 7200, 10800,
];

/// A named profile from NSS KeyboardLedProfiles. UI fields and type labels are
/// carried as JSON data when exporting from a template.
pub struct NssProfile(Value);

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NssData {
    name: String,
    ac_idle_timer: usize,
    dc_idle_timer: usize,
    pattern: Pattern,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Pattern {
    selected_effect: u32,
    selected_direction: u32,
    #[serde(rename = "ACBrightness")]
    ac_brightness: u8,
    #[serde(rename = "DCBrightness")]
    dc_brightness: u8,
    speed: u8,
    available_colors: Vec<PaletteColor>,
    keys: Vec<KeyColor>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PaletteColor {
    key: u32,
    value: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct KeyColor {
    index: usize,
    selected_color: String,
}

impl NssProfile {
    pub fn read(reader: impl Read) -> Result<Self> {
        let document = Self(read_json(reader)?);
        document.data()?;
        Ok(document)
    }

    fn data(&self) -> Result<NssData> {
        Ok(serde_json::from_value(self.0.clone())?)
    }

    pub fn to_profile(&self, layout: KeyLayout) -> Result<Profile> {
        let data = self.data()?;
        let pattern = data.pattern;
        let mut colors = BTreeMap::new();
        for slot in pattern.available_colors {
            if colors
                .insert(slot.key, decode_color(&slot.value)?)
                .is_some()
            {
                return Err(error("NSS 调色板索引重复"));
            }
        }
        let effect = Effect::from_console(pattern.selected_effect)?;
        let mut key_colors = Vec::new();
        if effect == Effect::UserMode {
            key_colors.resize(KEY_COUNT, [0; 3]);
            for key in pattern.keys {
                for &slot in slots(layout, key.index)? {
                    key_colors[slot] = decode_color(&key.selected_color)?;
                }
            }
        }
        let profile = Profile {
            effect,
            direction: Direction::from_console(pattern.selected_direction)?,
            speed: pattern.speed,
            brightness_ac: pattern.ac_brightness,
            brightness_dc: pattern.dc_brightness,
            idle_ac: timer_seconds(data.ac_idle_timer)?,
            idle_dc: timer_seconds(data.dc_idle_timer)?,
            palette: colors.into_values().collect(),
            key_colors,
            power: true,
        };
        profile.validate()?;
        Ok(profile)
    }

    /// Write a new NSS file using the source document's UI metadata.
    /// The template must be for the selected effect and the same layout.
    pub fn export(
        &self,
        profile: &Profile,
        layout: KeyLayout,
        name: &str,
        mut writer: impl Write,
    ) -> Result<()> {
        profile.validate()?;
        let data = self.data()?;
        if data.name.is_empty() || name.trim().is_empty() {
            return Err(error("NSS 配置需要名称"));
        }
        let template = self.to_profile(layout)?;
        if template.effect != profile.effect || template.palette.len() != profile.palette.len() {
            return Err(error("NSS 导出需要相同灯效、调色板槽数的原厂模板"));
        }
        if !profile.power {
            return Err(error("NSS 命名配置没有总电源字段，请用 power 命令单独设置"));
        }
        let mut document = self.0.clone();
        document["Name"] = json!(name);
        document["AcIdleTimer"] = json!(timer_tick(profile.idle_ac)?);
        document["DcIdleTimer"] = json!(timer_tick(profile.idle_dc)?);
        let pattern = &mut document["Pattern"];
        let direction = json!(profile.direction as u32);
        if !pattern["Directions"]
            .as_array()
            .is_some_and(|values| values.contains(&direction))
        {
            return Err(error("NSS 模板不支持此方向"));
        }
        pattern["SelectedDirection"] = direction;
        pattern["Speed"] = json!(profile.speed);
        pattern["ACBrightness"] = json!(profile.brightness_ac);
        pattern["DCBrightness"] = json!(profile.brightness_dc);

        let mut palette_keys: Vec<_> = data
            .pattern
            .available_colors
            .iter()
            .map(|c| c.key)
            .collect();
        palette_keys.sort_unstable();
        let palette: BTreeMap<_, _> = palette_keys
            .into_iter()
            .zip(profile.palette.iter().copied())
            .collect();
        for entry in pattern["AvailableColors"]
            .as_array_mut()
            .expect("validated palette array")
        {
            let index = entry["Key"].as_u64().expect("validated palette key") as u32;
            entry["Value"] = json!(encode_color(palette[&index]));
        }
        if let Some(selected) = pattern["SelectedColors"].as_array_mut() {
            for entry in selected {
                if let Some(color) = entry["Index"]
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .and_then(|n| palette.get(&n))
                {
                    entry["SelectedColor"] = json!(encode_color(*color));
                }
            }
        }
        if profile.effect == Effect::UserMode {
            let blank = [[0; 3]; KEY_COUNT];
            let target = if profile.key_colors.is_empty() {
                &blank[..]
            } else {
                &profile.key_colors
            };
            let mut reconstructed = blank;
            for (entry, key) in pattern["Keys"]
                .as_array_mut()
                .expect("validated key array")
                .iter_mut()
                .zip(data.pattern.keys)
            {
                let slots = slots(layout, key.index)?;
                let color = target[slots[0]];
                for &slot in slots {
                    if target[slot] != color {
                        return Err(error("NSS 一个物理键的多个槽不能保存不同颜色"));
                    }
                    reconstructed[slot] = color;
                }
                entry["SelectedColor"] = json!(encode_color(color));
            }
            if reconstructed != target {
                return Err(error("NSS 模板缺少目标逐键槽"));
            }
        }
        serde_json::to_writer_pretty(&mut writer, &document)?;
        writeln!(writer)?;
        Ok(())
    }
}

/// Named groups stored by NSS in KeyGroups.json.
#[derive(Default)]
pub struct NssKeyGroups(Vec<Group>);

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Group {
    name: String,
    selected_key_ids: Vec<usize>,
}

impl NssKeyGroups {
    pub fn read(reader: impl Read) -> Result<Self> {
        Ok(Self(serde_json::from_value(read_json(reader)?)?))
    }

    pub fn groups(&self) -> impl Iterator<Item = (&str, &[usize])> {
        self.0
            .iter()
            .map(|g| (g.name.as_str(), g.selected_key_ids.as_slice()))
    }

    pub fn set(&mut self, layout: KeyLayout, name: &str, keys: &[&str]) -> Result<()> {
        if name.trim().is_empty() || keys.is_empty() {
            return Err(error("按键分组需要名称和至少一个物理键"));
        }
        let mut selected_key_ids = Vec::with_capacity(keys.len());
        for key in keys {
            let index = layout
                .keys()
                .iter()
                .position(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
                .ok_or_else(|| error(format!("该布局没有名为 {key} 的键")))?;
            selected_key_ids.push(index);
        }
        selected_key_ids.sort_unstable();
        selected_key_ids.dedup();
        let group = Group {
            name: name.into(),
            selected_key_ids,
        };
        if let Some(existing) = self
            .0
            .iter_mut()
            .find(|g| g.name.eq_ignore_ascii_case(name))
        {
            *existing = group;
        } else {
            self.0.push(group);
        }
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        let index = self
            .0
            .iter()
            .position(|g| g.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| error(format!("未找到 NSS 按键分组 {name}")))?;
        self.0.remove(index);
        Ok(())
    }

    pub fn write(&self, mut writer: impl Write) -> Result<()> {
        let groups: Vec<_> = self
            .0
            .iter()
            .map(|g| {
                json!({
                    "$type": "KeyGroupStorageData",
                    "Name": g.name,
                    "SelectedKeyIds": g.selected_key_ids,
                })
            })
            .collect();
        serde_json::to_writer_pretty(&mut writer, &groups)?;
        writeln!(writer)?;
        Ok(())
    }

    pub fn paint(
        &self,
        profile: &mut Profile,
        layout: KeyLayout,
        name: &str,
        color: Rgb,
    ) -> Result<()> {
        profile.validate()?;
        let group = self
            .0
            .iter()
            .find(|g| g.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| error(format!("未找到 NSS 按键分组 {name}")))?;
        let mut indices = Vec::new();
        for &index in &group.selected_key_ids {
            indices.extend_from_slice(slots(layout, index)?);
        }
        profile.key_colors.resize(KEY_COUNT, [0; 3]);
        for index in indices {
            profile.key_colors[index] = color;
        }
        profile.effect = Effect::UserMode;
        Ok(())
    }
}

/// Apply a named group from NSS KeyGroups.json to a profile.
pub fn paint_nss_group(
    profile: &mut Profile,
    layout: KeyLayout,
    reader: impl Read,
    name: &str,
    color: Rgb,
) -> Result<()> {
    NssKeyGroups::read(reader)?.paint(profile, layout, name, color)
}

fn read_json(mut reader: impl Read) -> Result<Value> {
    let mut text = String::new();
    reader.read_to_string(&mut text)?;
    Ok(serde_json::from_str(text.trim_start_matches('\u{feff}'))?)
}

fn slots(layout: KeyLayout, index: usize) -> Result<&'static [usize]> {
    layout
        .keys()
        .get(index)
        .map(|entry| entry.1)
        .ok_or_else(|| error(format!("键位编号 {index} 超出 {layout:?} 布局")))
}

fn timer_seconds(tick: usize) -> Result<u32> {
    TIMER_SECONDS
        .get(tick)
        .copied()
        .ok_or_else(|| error(format!("无效 NSS 空闲档位 {tick}")))
}

fn timer_tick(seconds: u32) -> Result<usize> {
    TIMER_SECONDS
        .iter()
        .position(|&n| n == seconds)
        .ok_or_else(|| error(format!("NSS 没有 {seconds} 秒空闲档位")))
}

fn decode_color(value: &str) -> Result<Rgb> {
    let hex = value
        .strip_prefix('#')
        .filter(|s| s.len() == 8 && s.is_ascii())
        .ok_or_else(|| error(format!("预期 NSS #AARRGGBB 颜色，实际为 {value}")))?;
    let argb = u32::from_str_radix(hex, 16)?;
    let [_, r, g, b] = argb.to_be_bytes();
    Ok(if argb == 0xff80_8080 {
        [0; 3]
    } else {
        [r, g, b]
    })
}

fn encode_color([r, g, b]: Rgb) -> String {
    // Opaque WPF Gray is the NSS off sentinel. A different alpha preserves LED gray.
    match [r, g, b] {
        [0, 0, 0] => "#FF808080".into(),
        [128, 128, 128] => "#FE808080".into(),
        _ => format!("#FF{r:02X}{g:02X}{b:02X}"),
    }
}

fn error(message: impl Into<String>) -> Box<dyn std::error::Error> {
    HidError(message.into()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> NssProfile {
        let enter = KeyLayout::Iso
            .keys()
            .iter()
            .position(|&(name, _)| name == "ENTER")
            .unwrap();
        NssProfile(json!({
            "$type": "SerializableKeyboardProfile", "Name": "KC",
            "AcIdleTimer": 4, "DcIdleTimer": 7,
            "Pattern": {
                "$type": "SerializableKeyboardPattern", "SelectedEffect": 11,
                "SelectedDirection": 0, "Directions": [0], "Speed": 5,
                "ACBrightness": 70, "DCBrightness": 20, "EditingEnabled": true,
                "AvailableColors": [{"Key": 2, "Value": "#FF808080"}, {"Key": 0, "Value": "#FFFF0000"}],
                "SelectedColors": [{"$type": "SerializableColorPickerColors", "Index": 2, "SelectedColor": "#FF808080", "Enabled": true}],
                "Keys": [{"$type": "KeyColorData", "Index": enter, "SelectedColor": "#FF123456", "Enabled": false}]
            }
        }))
    }

    #[test]
    fn original_template_round_trips_timers_palette_and_key_colors() {
        let template = fixture();
        let mut profile = template.to_profile(KeyLayout::Iso).unwrap();
        assert_eq!((profile.idle_ac, profile.idle_dc), (30, 120));
        assert_eq!(profile.palette, vec![[255, 0, 0], [0, 0, 0]]);
        profile.palette[1] = [128; 3];
        profile.idle_dc = 45;
        profile
            .paint_key(KeyLayout::Iso, "ENTER", [1, 2, 3])
            .unwrap();
        let mut bytes = Vec::new();
        template
            .export(&profile, KeyLayout::Iso, "Edited", &mut bytes)
            .unwrap();
        let exported = NssProfile::read(bytes.as_slice()).unwrap();
        assert_eq!(exported.to_profile(KeyLayout::Iso).unwrap(), profile);
        assert_eq!(exported.0["Name"], "Edited");
        assert_eq!(exported.0["Pattern"]["EditingEnabled"], true);
        assert_eq!(
            exported.0["Pattern"]["SelectedColors"][0]["SelectedColor"],
            "#FE808080"
        );
    }

    #[test]
    fn nss_export_rejects_slot_colors_a_template_cannot_represent() {
        let template = fixture();
        let mut profile = template.to_profile(KeyLayout::Iso).unwrap();
        profile.key_colors[0] = [1, 2, 3];
        let mut bytes = Vec::new();
        assert!(
            template
                .export(&profile, KeyLayout::Iso, "Edited", &mut bytes)
                .is_err()
        );
        assert!(bytes.is_empty());
    }

    #[test]
    fn key_group_validation_leaves_profile_unchanged_on_bad_index() {
        let mut profile = Profile::default();
        let invalid =
            br#"[{"$type":"KeyGroupStorageData","Name":"Group","SelectedKeyIds":[0,999]}]"#;
        assert!(
            paint_nss_group(
                &mut profile,
                KeyLayout::Us,
                &invalid[..],
                "Group",
                [1, 2, 3]
            )
            .is_err()
        );
        assert_eq!(profile, Profile::default());
        let group = br#"[{"$type":"KeyGroupStorageData","Name":"Group","SelectedKeyIds":[0,1]}]"#;
        paint_nss_group(&mut profile, KeyLayout::Us, &group[..], "group", [1, 2, 3]).unwrap();
        assert_eq!(profile.effect, Effect::UserMode);
        assert_eq!(&profile.key_colors[..3], &[[1, 2, 3], [1, 2, 3], [0; 3]]);
    }
}
