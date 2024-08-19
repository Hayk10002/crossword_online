use crossword_generator::word::Word;
use serde::{Deserialize, Serialize};

use super::color_rgba::ColorRGBA;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Serialize, Deserialize)]
pub struct Theme
{
    pub color_dark: ColorRGBA,
    pub color_normal: ColorRGBA,
    pub color_light: ColorRGBA,
    pub color_error_dark: ColorRGBA,
    pub color_error_normal: ColorRGBA,
    pub color_error_light: ColorRGBA,   
}

impl Theme
{
    pub fn new() -> Theme
    {
        Theme
        {
            color_dark: ColorRGBA::opaque(84, 84, 84),
            color_normal: ColorRGBA::opaque(125, 125, 125),
            color_light: ColorRGBA::opaque(156, 156, 156),
            color_error_dark: ColorRGBA::opaque(255, 84, 84),
            color_error_normal: ColorRGBA::opaque(255, 125, 125),
            color_error_light: ColorRGBA::opaque(255, 156, 156),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Serialize, Deserialize)]
pub struct PlaygroundStyleSettings
{
    pub gap: usize,
    pub border_radius: usize,
    pub cell_size: usize,
    pub font_size: usize,
    pub word_border_dist_from_cell_wall: usize,
    pub word_border_width: usize,
    pub word_border_radius: usize,
    pub between_word_width: usize,
    pub between_word_radius: usize,
    pub theme: Theme,
}

impl PlaygroundStyleSettings
{
    pub fn new() -> PlaygroundStyleSettings
    {
        PlaygroundStyleSettings
        {
            gap: 20,
            border_radius: 20,
            cell_size: 200,
            font_size: 80,
            word_border_dist_from_cell_wall: 20,
            word_border_width: 8,
            word_border_radius: 40,
            between_word_width: 8,
            between_word_radius: 20,
            theme: Theme::new(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Serialize, Deserialize)]
pub struct WordStyleSettings
{
    pub theme: Theme,
}

impl WordStyleSettings
{
    pub fn new() -> WordStyleSettings
    {
        WordStyleSettings
        {
            theme: Theme::new(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Serialize, Deserialize)]
pub struct StyleSettings
{
    pub word_style_settings: WordStyleSettings,
    pub playground_style_settings: PlaygroundStyleSettings,
}

impl StyleSettings
{
    pub fn new() -> StyleSettings
    {
        StyleSettings
        {
            word_style_settings: WordStyleSettings::new(),
            playground_style_settings: PlaygroundStyleSettings::new(),
        }
    }
}