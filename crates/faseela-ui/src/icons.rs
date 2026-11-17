//! Prepare an application's embedded SVG icons once, then share their bytes with egui.
//!
//! The application owns its assets, tool-name mapping, theme, sizing and button behavior.
//! Install `egui_extras`' SVG image loader on the context before drawing an icon. This module
//! does not parse SVG files, read from disk, or install global UI state.
//!
//! Adapted from <https://github.com/storytold/craft-libs> (MIT OR Apache-2.0).

use std::collections::HashMap;
use std::sync::Arc;

use egui::{ImageSource, load::Bytes};

/// Literal substitutions for trusted, embedded, monochrome SVG assets.
#[derive(Clone, Copy, Debug)]
pub struct SvgStyle {
    /// Replacement for `currentColor`, usually `white` or `#ffffff` for draw-time tinting.
    pub current_color: &'static str,
    /// Replacement width for the catalog's default two-unit stroke; `None` preserves it.
    pub stroke_width: Option<&'static str>,
}

/// An immutable catalog of prepared SVG bytes.
pub struct SvgIconSet {
    icons: HashMap<&'static str, Arc<[u8]>>,
    uri_prefix: &'static str,
    fallback: &'static str,
}

impl SvgIconSet {
    /// Prepare a finite, compile-time catalog. The last entry wins for duplicate names.
    ///
    /// `uri_prefix` includes the scheme and trailing separator, e.g. `bytes://my-app/icons/`.
    /// `fallback` is an icon name in the same catalog. If neither the requested icon nor the
    /// fallback exists, `source` returns empty bytes for egui to report as a load error.
    /// Invalid UTF-8 is replaced lossily. This API is for trusted embedded assets, not
    /// user-supplied SVG documents.
    pub fn new(
        icons: &'static [(&'static str, &'static [u8])],
        style: SvgStyle,
        uri_prefix: &'static str,
        fallback: &'static str,
    ) -> Self {
        let stroke = style
            .stroke_width
            .map(|width| format!("stroke-width=\"{width}\""));
        let icons = icons
            .iter()
            .map(|(name, bytes)| {
                let mut svg =
                    String::from_utf8_lossy(bytes).replace("currentColor", style.current_color);
                if let Some(stroke) = &stroke {
                    svg = svg.replace("stroke-width=\"2\"", stroke);
                }
                (*name, Arc::from(svg.into_bytes()))
            })
            .collect();
        Self {
            icons,
            uri_prefix,
            fallback,
        }
    }

    /// Whether the catalog contains `name`, without applying the fallback.
    pub fn contains(&self, name: &str) -> bool {
        self.icons.contains_key(name)
    }

    /// Borrow the embedded name, for applications that map dynamic tool names to static names.
    pub fn canonical_name(&self, name: &str) -> Option<&'static str> {
        self.icons.get_key_value(name).map(|(key, _)| *key)
    }

    /// A source suitable for `egui::Image::new`, with the requested name in the URI.
    pub fn source(&self, name: &str) -> ImageSource<'static> {
        let bytes = self
            .icons
            .get(name)
            .or_else(|| self.icons.get(self.fallback))
            .cloned()
            .unwrap_or_default();
        ImageSource::Bytes {
            uri: format!("{}{name}.svg", self.uri_prefix).into(),
            bytes: Bytes::Shared(bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ICONS: &[(&str, &[u8])] = &[
        ("check", br#"<svg stroke="currentColor" stroke-width="2"/>"#),
        ("plain", br#"<svg stroke="currentColor"/>"#),
    ];

    #[test]
    fn applies_color_and_stroke_to_embedded_assets() {
        let icons = SvgIconSet::new(
            ICONS,
            SvgStyle {
                current_color: "#ffffff",
                stroke_width: Some("1.5"),
            },
            "bytes://faseela/icons/",
            "check",
        );
        let ImageSource::Bytes {
            uri,
            bytes: Bytes::Shared(bytes),
        } = icons.source("check")
        else {
            panic!("expected shared embedded SVG bytes");
        };
        assert_eq!(uri.as_ref(), "bytes://faseela/icons/check.svg");
        let svg = std::str::from_utf8(&bytes).expect("generated SVG is UTF-8");
        assert!(svg.contains("#ffffff"));
        assert!(svg.contains("stroke-width=\"1.5\""));
    }

    #[test]
    fn missing_icons_use_fallback_but_keep_requested_uri() {
        let icons = SvgIconSet::new(
            ICONS,
            SvgStyle {
                current_color: "white",
                stroke_width: None,
            },
            "bytes://faseela/",
            "check",
        );
        assert!(!icons.contains("missing"));
        assert_eq!(icons.canonical_name("check"), Some("check"));
        assert_eq!(icons.canonical_name("missing"), None);
        let ImageSource::Bytes { uri, .. } = icons.source("missing") else {
            panic!("expected an embedded SVG source");
        };
        assert_eq!(uri.as_ref(), "bytes://faseela/missing.svg");
    }
}
