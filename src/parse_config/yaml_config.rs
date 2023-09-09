/*
 * This file is borrowed from espanso
 *
 * Copyright (C) 2019-2023 Federico Terzi
 *
 */

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;
use std::convert::TryFrom;

use super::ParsedConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct YAMLConfig {
    #[serde(default)]
    pub label: Option<String>,

    #[serde(default)]
    pub backend: Option<String>,

    #[serde(default)]
    pub enable: Option<bool>,

    #[serde(default)]
    pub clipboard_threshold: Option<usize>,

    #[serde(default)]
    pub pre_paste_delay: Option<usize>,

    #[serde(default)]
    pub toggle_key: Option<String>,

    #[serde(default)]
    pub auto_restart: Option<bool>,

    #[serde(default)]
    pub preserve_clipboard: Option<bool>,

    #[serde(default)]
    pub restore_clipboard_delay: Option<usize>,

    #[serde(default)]
    pub paste_shortcut_event_delay: Option<usize>,

    #[serde(default)]
    pub paste_shortcut: Option<String>,

    #[serde(default)]
    pub disable_x11_fast_inject: Option<bool>,

    #[serde(default)]
    pub inject_delay: Option<usize>,

    #[serde(default)]
    pub key_delay: Option<usize>,

    #[serde(default)]
    pub backspace_delay: Option<usize>,

    #[serde(default)]
    pub evdev_modifier_delay: Option<usize>,

    #[serde(default)]
    pub word_separators: Option<Vec<String>>,

    #[serde(default)]
    pub backspace_limit: Option<usize>,

    #[serde(default)]
    pub apply_patch: Option<bool>,

    #[serde(default)]
    pub keyboard_layout: Option<Mapping>,

    #[serde(default)]
    pub search_trigger: Option<String>,

    #[serde(default)]
    pub search_shortcut: Option<String>,

    #[serde(default)]
    pub undo_backspace: Option<bool>,

    #[serde(default)]
    pub show_notifications: Option<bool>,

    #[serde(default)]
    pub show_icon: Option<bool>,

    #[serde(default)]
    pub post_form_delay: Option<usize>,

    #[serde(default)]
    pub post_search_delay: Option<usize>,

    #[serde(default)]
    pub secure_input_notification: Option<bool>,

    #[serde(default)]
    pub emulate_alt_codes: Option<bool>,

    #[serde(default)]
    pub win32_exclude_orphan_events: Option<bool>,

    #[serde(default)]
    pub win32_keyboard_layout_cache_interval: Option<i64>,

    #[serde(default)]
    pub x11_use_xclip_backend: Option<bool>,

    #[serde(default)]
    pub x11_use_xdotool_backend: Option<bool>,

    // Include/Exclude
    #[serde(default)]
    pub includes: Option<Vec<String>>,

    #[serde(default)]
    pub excludes: Option<Vec<String>>,

    #[serde(default)]
    pub extra_includes: Option<Vec<String>>,

    #[serde(default)]
    pub extra_excludes: Option<Vec<String>>,

    #[serde(default)]
    pub use_standard_includes: Option<bool>,

    // Filters
    #[serde(default)]
    pub filter_title: Option<String>,

    #[serde(default)]
    pub filter_class: Option<String>,

    #[serde(default)]
    pub filter_exec: Option<String>,

    #[serde(default)]
    pub filter_os: Option<String>,
}

impl YAMLConfig {
    pub fn parse_from_str(yaml: &str) -> Result<Self> {
        // Because an empty string is not valid YAML but we want to support it anyway
        if is_yaml_empty(yaml) {
            return Ok(serde_yaml::from_str(
                "arbitrary_field_that_will_not_block_the_parser: true",
            )?);
        }

        Ok(serde_yaml::from_str(yaml)?)
    }
}

impl TryFrom<YAMLConfig> for ParsedConfig {
    type Error = anyhow::Error;

    fn try_from(yaml_config: YAMLConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            label: yaml_config.label,
            backend: yaml_config.backend,
            enable: yaml_config.enable,
            clipboard_threshold: yaml_config.clipboard_threshold,
            auto_restart: yaml_config.auto_restart,
            toggle_key: yaml_config.toggle_key,
            preserve_clipboard: yaml_config.preserve_clipboard,
            paste_shortcut: yaml_config.paste_shortcut,
            disable_x11_fast_inject: yaml_config.disable_x11_fast_inject,
            inject_delay: yaml_config.inject_delay,
            key_delay: yaml_config.key_delay.or(yaml_config.backspace_delay),
            evdev_modifier_delay: yaml_config.evdev_modifier_delay,
            word_separators: yaml_config.word_separators,
            backspace_limit: yaml_config.backspace_limit,
            apply_patch: yaml_config.apply_patch,
            keyboard_layout: yaml_config.keyboard_layout.map(|mapping| {
                mapping
                    .into_iter()
                    .filter_map(|(key, value)| {
                        if let (Some(key), Some(value)) = (key.as_str(), value.as_str()) {
                            Some((key.to_string(), value.to_string()))
                        } else {
                            None
                        }
                    })
                    .collect()
            }),
            search_trigger: yaml_config.search_trigger,
            search_shortcut: yaml_config.search_shortcut,
            undo_backspace: yaml_config.undo_backspace,

            show_icon: yaml_config.show_icon,
            show_notifications: yaml_config.show_notifications,
            secure_input_notification: yaml_config.secure_input_notification,

            pre_paste_delay: yaml_config.pre_paste_delay,
            restore_clipboard_delay: yaml_config.restore_clipboard_delay,
            paste_shortcut_event_delay: yaml_config.paste_shortcut_event_delay,
            post_form_delay: yaml_config.post_form_delay,
            post_search_delay: yaml_config.post_search_delay,

            emulate_alt_codes: yaml_config.emulate_alt_codes,

            win32_exclude_orphan_events: yaml_config.win32_exclude_orphan_events,
            win32_keyboard_layout_cache_interval: yaml_config.win32_keyboard_layout_cache_interval,
            x11_use_xclip_backend: yaml_config.x11_use_xclip_backend,
            x11_use_xdotool_backend: yaml_config.x11_use_xdotool_backend,

            use_standard_includes: yaml_config.use_standard_includes,
            includes: yaml_config.includes,
            extra_includes: yaml_config.extra_includes,
            excludes: yaml_config.excludes,
            extra_excludes: yaml_config.extra_excludes,

            filter_class: yaml_config.filter_class,
            filter_exec: yaml_config.filter_exec,
            filter_os: yaml_config.filter_os,
            filter_title: yaml_config.filter_title,
        })
    }
}

pub fn is_yaml_empty(yaml: &str) -> bool {
    for line in yaml.lines() {
        let trimmed_line = line.trim();
        if !trimmed_line.starts_with('#') && !trimmed_line.is_empty() {
            return false;
        }
    }

    true
}
