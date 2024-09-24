use crossword_generator::{crossword::WordCompatibilityError, placed_word::PlacedWord, traits::{CrosswordChar, CrosswordString}, word::{Direction, Position}};
use stylist::{css, yew::styled_component};
use yew::prelude::*;

use crate::utils::{color_rgba::ColorRGBA, settings::{PlaygroundStyleSettings, StyleSettings, Theme}};

use super::playground_component::PlaygroundWordState;

#[derive(Properties, PartialEq)]
pub struct PlaygroundCellComponentProperties<CharT: CrosswordChar>
{
    pub position: Position,
    pub word_ids: Vec<usize>,
    pub character: Option<CharT>,
    #[prop_or(PlaygroundWordState::Normal)]
    pub state: PlaygroundWordState,
    #[prop_or(Callback::noop())]
    pub on_invert_word_direction: Callback<()>,
    #[prop_or(Callback::noop())]
    pub on_select: Callback<bool>,
}

#[styled_component]
pub fn PlaygroundCellComponent<CharT: CrosswordChar + ToHtml>(PlaygroundCellComponentProperties{position: pos, word_ids, character, state, on_invert_word_direction, on_select}: &PlaygroundCellComponentProperties<CharT>) -> Html
{
    let StyleSettings { word_style_settings: _, playground_style_settings } = use_context::<StyleSettings>().expect("No style provided");
    let PlaygroundStyleSettings 
        { 
            gap, 
            cell_size, 
            border_radius, 
            font_size, 
            word_border_dist_from_cell_wall: _, 
            word_border_width: _, 
            word_border_radius: _,
            between_word_width: _,
            between_word_radius: _,
            theme
        } = playground_style_settings;
    
    let Theme
        {
            color_dark: _,
            color_error_dark: _,
            color_normal,
            color_error_normal,
            color_light,
            color_error_light,
        } = theme;
    
    let words_visible_when_hovered = word_ids.into_iter().map(|id| 
        css!(
            :hover ~ #${format!("word{}", id)} 
            { 
                visibility: visible;
            }
        )
    ).collect::<Vec<_>>();

    let (background_color, hover_background_color) = 
        if let Some(_) = &character 
            { (color_light, color_normal) } 
        else 
            { (color_error_light, color_error_normal) };

    let hover_effect_when_selected = (state == &PlaygroundWordState::Selected).then_some(css!
        (
            background-color: ${hover_background_color};
        )
    );

    html!
    {
        <div class={classes!("playground_cell",
            css!
            (
                position: absolute;
                background-color: ${background_color};
                width: ${cell_size}px;
                height: ${cell_size}px;
                border-radius: ${border_radius}px;
                font-size: ${font_size}px;
                text-align: center;
                align-content: center;
                color: white; 
                cursor: default;
                user-select: none;
                pointer-events: inherit;

                :hover
                {  
                    background-color: ${hover_background_color};
                }
            ),
            words_visible_when_hovered,
            hover_effect_when_selected,
            css!(
                left: ${pos.x as isize * cell_size as isize + pos.x as isize * gap as isize}px;
                top: ${pos.y as isize * cell_size as isize + pos.y as isize * gap as isize}px;
            )
        )} 
        draggable={ (state == &PlaygroundWordState::Selected).to_string() }
        onclick={on_select.reform(|event: MouseEvent| event.ctrl_key())}>
            { character }
        </div>
    }
}



#[derive(Properties, PartialEq)]
pub struct PlaygroundBetweenCellComponentProperties
{
    pub position: Position,
    pub direction: Direction,
    pub word_ids: Vec<usize>,
    pub draggable: bool,
    #[prop_or(Callback::noop())]
    pub on_select: Callback<bool>,
}

#[styled_component]
pub fn PlaygroundBetweenCellComponent(PlaygroundBetweenCellComponentProperties{position: pos, direction: dir, word_ids, draggable,  on_select}: &PlaygroundBetweenCellComponentProperties) -> Html
{
    let StyleSettings { word_style_settings: _, playground_style_settings } = use_context::<StyleSettings>().expect("No style provided");
    let PlaygroundStyleSettings 
        { 
            gap, 
            cell_size, 
            border_radius: _, 
            font_size: _, 
            word_border_dist_from_cell_wall: _, 
            word_border_width: _, 
            word_border_radius: _,
            between_word_width: _,
            between_word_radius: _,
            theme: _,
        } = playground_style_settings;

    
    let words_visible_when_hovered = word_ids.into_iter().map(|id| 
        css!(
            :hover ~ #${format!("word{}", id)} 
            { 
                visibility: visible;
            }
    )).collect::<Vec<_>>();

    
    let (width, height) = match dir
    {
        Direction::Right => (gap, cell_size),
        Direction::Down => (cell_size, gap),
    };

    html!
    {
        <div class={classes!("between_cell",
            css!
            (
                position: absolute;
                background-color: transparent;
                width: ${width}px;
                height: ${height}px;
                cursor: default;
                user-select: none;
            ),
            words_visible_when_hovered,
            css!(
                left: ${(pos.x as isize + (*dir == Direction::Right) as isize) * cell_size as isize + pos.x as isize * gap as isize}px;
                top: ${(pos.y as isize + (*dir == Direction::Down) as isize) * cell_size as isize + pos.y as isize * gap as isize}px;
            )
        )}
        draggable={draggable.to_string()}
        onclick={on_select.reform(|event: MouseEvent| event.ctrl_key())}/>
    }
}


#[derive(Properties, PartialEq)]
pub struct PlaygroundWordComponentProperties
{
    pub position: Position,
    pub width: usize,
    pub height: usize,
    pub id: usize,
    pub error_exists: bool,
    #[prop_or(PlaygroundWordState::Normal)]
    pub state: PlaygroundWordState,
}

#[styled_component]
pub fn PlaygroundWordComponent(PlaygroundWordComponentProperties{position: pos, width, height, id, error_exists, state}: &PlaygroundWordComponentProperties) -> Html
{
    let StyleSettings { word_style_settings: _, playground_style_settings } = use_context::<StyleSettings>().expect("No style provided");
    let PlaygroundStyleSettings 
        { 
            gap, 
            cell_size, 
            border_radius: _, 
            font_size: _, 
            word_border_dist_from_cell_wall, 
            word_border_width, 
            word_border_radius,
            between_word_width: _,
            between_word_radius: _,
            theme,
        } = playground_style_settings;
    
    let Theme
        {
            color_dark,
            color_error_dark,
            color_normal: _,
            color_error_normal: _,
            color_light: _,
            color_error_light: _,
        } = theme;

    
    let word_red_when_errors = error_exists.then_some(
        css!(
            visibility: visible;
            border-color: ${color_error_dark};
    ));

    let word_visible_when_selected = (state == &PlaygroundWordState::Selected).then_some(
        css!
        (
            visibility: visible;
        )
    );

    html!
    {
        <div id={format!("word{}", id)} 
            class={classes!("word_outline", 
            css!
            (
                position: absolute;
                background-color: transparent;
                border: ${word_border_width}px solid${" "}${color_dark};
                border-radius: ${word_border_radius}px;
                box-sizing: border-box;
                pointer-events: none;
                visibility: hidden;
                user-select: none;
            ),
            word_red_when_errors,
            word_visible_when_selected,
            css!
            (
                left: ${pos.x as isize * (cell_size + gap) as isize + word_border_dist_from_cell_wall as isize}px;
                top: ${pos.y as isize * (cell_size + gap) as isize + word_border_dist_from_cell_wall as isize}px;
                width: ${width * (cell_size + gap) - gap - 2 * word_border_dist_from_cell_wall}px;
                height: ${height * (cell_size + gap) - gap - 2 * word_border_dist_from_cell_wall}px;
            )
        )}/>
    }
}


#[derive(Properties, PartialEq)]
pub struct PlaygroundWordErrorOutlineComponentProperties
{
    pub position: Position,
    pub direction: Direction,
    pub length: usize,
    pub start: usize,
    pub end: usize,
}

#[function_component]
pub fn PlaygroundWordErrorOutlineComponent(&PlaygroundWordErrorOutlineComponentProperties{position: ref pos, direction: ref dir, length: w_len, start, end}: &PlaygroundWordErrorOutlineComponentProperties) -> Html
{
    let (w_width, w_height) = match dir
    {
        Direction::Right => (w_len, 1),
        Direction::Down => (1, w_len),
    };

    let StyleSettings { word_style_settings: _, playground_style_settings } = use_context::<StyleSettings>().expect("No style provided");
    let PlaygroundStyleSettings 
        { 
            gap, 
            cell_size, 
            border_radius: _, 
            font_size: _, 
            word_border_dist_from_cell_wall: _, 
            word_border_width: _, 
            word_border_radius: _,
            between_word_width,
            between_word_radius,
            theme,
        } = playground_style_settings;
    
    let Theme
        {
            color_dark: _,
            color_error_dark: _,
            color_normal: _,
            color_error_normal,
            color_light: _,
            color_error_light: _,
        } = theme;


    let html = match dir
    {
        Direction::Right => [(0usize, w_len - 1), (w_len, w_len), (w_len + 1, 2 * w_len), (2 * w_len + 1, 2 * w_len + 1)],
        Direction::Down => [(0, 0), (1, w_len), (w_len + 1, w_len + 1), (w_len + 2, 2 * w_len + 1)],
    }.into_iter().zip([0, 1, 2, 3]).flat_map(move |((side_start, side_end), side_direction)| // direction 0 top, 1 right, 2 bottom, 3 left
    {
        let check_round_order = |a, b, c| (a && b) || (b && c) || (c && a);
        
        (side_start..=side_end).into_iter().filter_map(move |curr|
        {
            if !check_round_order(start <= curr, curr <= end, end < start) { return None; }
            let start_type = if curr == start {2} else if curr == side_start {1} else {0};
            let end_type = if curr == end {2} else if curr == side_end {1} else {0};

            let div_start = match start_type
            {
                0 => 0,
                1 => between_word_radius, 
                2 => (cell_size + gap) / 2,
                _ => unreachable!()
            };

            let div_end = match end_type
            {
                0 => cell_size + gap,
                1 => cell_size + gap - between_word_radius, 
                2 => (cell_size + gap) / 2,
                _ => unreachable!()
            };

            let div_start_color = if start_type == 2 { ColorRGBA{ a: 0, ..color_error_normal } } else { color_error_normal };
            let div_end_color = if end_type == 2 { ColorRGBA{ a: 0, ..color_error_normal } } else { color_error_normal };

            let (div_pos_x, div_pos_y) = match side_direction 
            {
                0 => ((pos.x as isize + (curr - side_start) as isize) * (cell_size + gap) as isize - gap as isize / 2 + div_start as isize, pos.y as isize * (cell_size + gap) as isize - gap as isize / 2 + between_word_width as isize / 2),
                1 => ((pos.x as isize + side_start as isize) * (cell_size + gap) as isize - gap as isize / 2 - between_word_width as isize / 2, (pos.y as isize + (curr - side_start) as isize) * (cell_size + gap) as isize - gap as isize / 2 + div_start as isize),
                2 => ((pos.x as isize + w_width as isize - (curr - side_start) as isize) * (cell_size + gap) as isize - gap as isize / 2 - div_start as isize, (pos.y as isize + w_height as isize) * (cell_size + gap) as isize - gap as isize / 2 - between_word_width as isize / 2),
                3 => (pos.x as isize * (cell_size + gap) as isize - gap as isize / 2 + between_word_width as isize / 2, (pos.y as isize + w_height as isize - (curr - side_start) as isize) * (cell_size + gap) as isize - gap as isize / 2 - div_start as isize),
                _ => unreachable!()
            };

            Some(html!
            {
                <div class={classes!("between_word_error_outline",
                    css!
                    (
                        position: absolute;
                        user-select: none;
                        width: ${between_word_width}px;
                        transform-origin: 0 0;
                    ),
                    css!
                    (
                        background-image: linear-gradient(to bottom, ${div_start_color}, ${div_end_color});
                        left: ${div_pos_x}px;
                        top: ${div_pos_y}px;
                        height: ${div_end - div_start}px;
                        transform: rotate(${-90 + side_direction * 90}deg);
                    )
                )}/>
            })
        }).chain(check_round_order(start <= side_end, side_end < end, end < start).then_some(
        {
            let svg_size = between_word_radius + between_word_width / 2;
            let (x, y) = match side_direction
            {
                0 => (w_width, 0),
                1 => (w_width, w_height),
                2 => (0, w_height),
                3 => (0, 0),
                _ => unreachable!()
            };

            let (x, y) = (x as isize + pos.x as isize, y as isize + pos.y as isize);
            let (x, y) = (x * (cell_size + gap) as isize - gap as isize / 2, y * (cell_size + gap) as isize - gap as isize / 2);
            html!
            {
                <svg class={classes!("between_word_error_outline",
                    css!
                    (
                        position: absolute;
                        user-select: none;
                        pointer-events: stroke;
                        transform-origin: 0 0;
                    ),
                    css!
                    (
                        left: ${x}px;
                        top: ${y}px;
                        transform: rotate(${side_direction * 90}deg) translate(${-(between_word_radius as isize)}px, ${-(between_word_width as isize) / 2}px);
                    )
                )}
                    width={svg_size.to_string()}
                    height={svg_size.to_string()}
                >
                    <path 
                        d={format!("M 0 {0} A {1} {1} 0 0 1 {2} {3}", between_word_width / 2, between_word_radius, svg_size - between_word_width / 2, svg_size)} 
                        stroke={color_error_normal.to_string()} 
                        stroke-width={between_word_width.to_string()}
                        fill="none"/>
                </svg>
            }
        }))
    });


    html!
    {
        <>
            { for html }
        </>
    }
}