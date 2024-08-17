#![allow(non_upper_case_globals)]
use std::cell::Cell;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::default;
use std::io::Empty;
use std::iter::{empty, once, repeat};

use crossword_generator::crossword::{Crossword, CrosswordError, WordCompatibilityError, WordCompatibilitySettings};
use crossword_generator::placed_word::PlacedWord;
use crossword_generator::traits::{CrosswordChar, CrosswordString};
use crossword_generator::word::{Direction, Position};
use gloo_console::log;
use html::IntoPropValue;
use itertools::Itertools;
use yew::prelude::*;
use stylist::{css, style};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use gloo_utils::format::JsValueSerdeExt;
use web_sys::{Element, HtmlElement};
use web_sys::CssStyleDeclaration;
use yew::virtual_dom::VNode;
use web_sys::WheelEvent;

use crate::utils::color_rgba::ColorRGBA;

use super::super::utils::weak_component_link::WeakComponentLink;

#[derive(PartialEq, Properties)]
pub struct PlaygroundComponentProps<CharT: CrosswordChar + ToHtml + 'static, StrT: CrosswordString<CharT> + 'static>
{
    pub words: Vec<PlacedWord<CharT, StrT>>,
    pub word_compatibility_settings: WordCompatibilitySettings,
    pub link: WeakComponentLink<PlaygroundComponent<CharT, StrT>>
}

pub enum PlaygroundComponentMessage<CharT: CrosswordChar, StrT: CrosswordString<CharT>>
{
    SetCrossword(Crossword<CharT, StrT>),
    SetWords(Vec<PlacedWord<CharT, StrT>>),
    AddWord(PlacedWord<CharT, StrT>),
    RemoveWord(StrT),

    Scroll(f32, f32),
    Zoom(f32),
}

pub struct PlaygroundComponent<CharT: CrosswordChar, StrT: CrosswordString<CharT>>
{
    words: Vec<PlacedWord<CharT, StrT>>,
    word_compatibility_settings: WordCompatibilitySettings,
    transform_x: f32,
    transform_y: f32,
    transform_zoom: f32,

    html: Vec<VNode>,
}

impl<CharT: CrosswordChar + ToHtml, StrT: CrosswordString<CharT>> PlaygroundComponent<CharT, StrT>
{

    fn recalculate_drawing_data(&mut self)
    {
        let mut word_data = self.words.iter().enumerate().map(|(i, w)| (w, (i, Vec::<(WordCompatibilityError, &PlacedWord<CharT, StrT>)>::default()))).collect::<HashMap<_, _>>();
        for words in self.words.iter().combinations(2)
        {
            if let Some(error) = self.word_compatibility_settings.word_compatibility_issue(words[0], words[1])
            {
                word_data.get_mut(words[0]).unwrap().1.push((error.clone(), words[1]));
                word_data.get_mut(words[1]).unwrap().1.push((error.clone(), words[0]));
            }           
        }


        let mut cell_data: HashMap<Position, (Vec<(&PlacedWord<CharT, StrT>, usize)>, Vec<(WordCompatibilityError, &PlacedWord<CharT, StrT>)>)> = HashMap::new();

        for (&w, _) in word_data.iter()
        {
            for i in 0..w.value.as_ref().len()
            {
                match &w.direction
                {
                    Direction::Right => cell_data.insert(Position { x: w.position.x + i as i16, y: w.position.y }, Default::default()), 
                    Direction::Down => cell_data.insert(Position { x: w.position.x, y: w.position.y  + i as i16}, Default::default()), 
                };
            }
        }

        for (&w, (_, errors)) in word_data.iter()
        {
            for i in 0..w.value.as_ref().len()
            {
                let cell = match &w.direction
                {
                    Direction::Right => cell_data.get_mut(&Position { x: w.position.x + i as i16, y: w.position.y }), 
                    Direction::Down => cell_data.get_mut(&Position { x: w.position.x, y: w.position.y  + i as i16}), 
                }.unwrap();
                cell.0.push((w, i));
                cell.1.extend(errors.iter().cloned());
            }
        }

        let mut between_cell_data: HashMap<(Position, Direction), (Vec<&PlacedWord<CharT, StrT>>, Vec<(WordCompatibilityError, &PlacedWord<CharT, StrT>)>)> = HashMap::new();

        for (&w, _) in word_data.iter()
        {
            for i in 0..w.value.as_ref().len() - 1
            {
                match &w.direction
                {
                    Direction::Right => between_cell_data.insert((Position { x: w.position.x + i as i16, y: w.position.y }, Direction::Right), Default::default()), 
                    Direction::Down => between_cell_data.insert((Position { x: w.position.x, y: w.position.y + i as i16}, Direction::Down), Default::default()), 
                };
            }
        }

        for (&w, (_, errors)) in word_data.iter()
        {
            for i in 0..w.value.as_ref().len() - 1
            {
                let between_cell = match &w.direction
                {
                    Direction::Right => between_cell_data.get_mut(&(Position { x: w.position.x + i as i16, y: w.position.y }, Direction::Right)), 
                    Direction::Down => between_cell_data.get_mut(&(Position { x: w.position.x, y: w.position.y + i as i16}, Direction::Down)), 
                }.unwrap();
                between_cell.0.push(w);
                between_cell.1.extend(errors.iter().cloned());
            }
        }

        let between_word_data = word_data.iter()
            .flat_map(|(&w, (_, errors))| 
                repeat(w)
                .zip(errors.iter())
                .map(|(w1, (e, w2))| (e, w1, *w2))
                .filter(|(e, _, _)| if let &&WordCompatibilityError::InvalidIntersection = e { false } else { true }))
            .map(|(e, w, w_o)| 
            {
                (e, match e
                {
                    WordCompatibilityError::CornerByCorner => 
                    {
                        let w_corner = (w_o.position.x < w.position.x) as usize * 2 + (w_o.position.y < w.position.y) as usize;// 0 leftup, 1 leftdown, 2 rightup, 3 rightdown

                        
                        (w, { let w_len = w.value.as_ref().len() as i16; match w_corner
                        {
                            0 => (w_len, w_len + 1),
                            1 => if w.direction == Direction::Down { (0, 1) } else { (w_len - 1, w_len) },
                            2 => if w.direction == Direction::Down { (w_len + 1, w_len + 2) } else { (2 * w_len, 2 * w_len + 1) },
                            3 => (2 * w_len + 1, 0),
                            _ => unreachable!()
                        }})
                    },

                    WordCompatibilityError::HeadByHead =>
                    {
                        let w_side = ((w.direction == Direction::Down) as usize * 2 + (w_o.position.x + w_o.position.y < w.position.x + w.position.y) as usize + 3) % 4; // 0 right, 1 up, 2 down, 3 left

                        (w, { let w_len = w.value.as_ref().len() as i16; match w_side
                        {
                            0 => (2 * w_len, 0),
                            1 => (w_len, w_len + 2),
                            2 => (2 * w_len + 1, 1),
                            3 => (w_len - 1, w_len + 1),
                            _ => unreachable!()
                        }})
                    },

                    WordCompatibilityError::SideBySide =>
                    {
                        let dir = (w.direction == Direction::Right) as usize;
                        let side_com = if dir == 1 { w_o.position.y < w.position.y } else { w_o.position.x < w.position.x } as usize;
                        let w_side = (dir * 2 + side_com + 3) % 4; // 0 right, 1 up, 2 down, 3 left

                        (w, { let w_len = w.value.as_ref().len() as i16; let w_o_len = w_o.value.as_ref().len() as i16; match w_side
                        {
                            0 => (max(w_len + 1, (w.position.y + w_len) - (w_o.position.y + w_o_len) + w_len + 1), min(w.position.y - w_o.position.y + 2 * w_len + 2, 2 * w_len + 2) % (2 * w_len + 2)),
                            1 => (max(w_len, (w.position.x + w_len) - (w_o.position.x + w_o_len) + w_len), min(w.position.x - w_o.position.x + 2 * w_len + 1, 2 * w_len + 1)),
                            2 => ((max(-1, w_o.position.x - w.position.x - 1) + 2 * w_len + 2) % (2 * w_len + 2), min((w_o.position.x + w_o_len) - (w.position.x + w_len) + w_len, w_len)),
                            3 => (max(0, w_o.position.y - w.position.y), min((w_o.position.y + w_o_len) - (w.position.y + w_len) + w_len as i16 + 1, w_len as i16 + 1)),
                            _ => unreachable!()
                        }})
                    },

                    WordCompatibilityError::SideByHead =>
                    {
                        let (w_width, w_height) = match w.direction
                        {
                            Direction::Right => (w.value.as_ref().len() as i16, 1),
                            Direction::Down => (1, w.value.as_ref().len() as i16),
                        };

                        let (w_o_width, w_o_height) = match w_o.direction
                        {
                            Direction::Right => (w_o.value.as_ref().len() as i16, 1),
                            Direction::Down => (1, w_o.value.as_ref().len() as i16),
                        };              

                        let w_right = (w.position.x >= w_o.position.x + w_o_width) as i16;
                        let w_up = (w.position.y + w_height <= w_o.position.y) as i16;
                        let w_down = (w.position.y >= w_o.position.y + w_o_height) as i16;
                        let w_left = (w.position.x + w_width <= w_o.position.x) as i16;
                        let w_side = 0 * w_right + 1 * w_up + 2 * w_down + 3 * w_left; // 0 right, 1 up, 2 down, 3 left

                        (w, { let w_len = w.value.as_ref().len() as i16; match (w_side, &w.direction)
                        {
                            (0, Direction::Right) => (2 * w_len, 0),
                            (0, Direction::Down) => (w.position.y - w_o.position.y + 2 * w_len, (w.position.y - w_o.position.y + 2 * w_len + 2) % (2 * w_len + 2)),
                            (1, Direction::Right) => (w.position.x - w_o.position.x + 2 * w_len - 1, w.position.x - w_o.position.x + 2 * w_len + 1),
                            (1, Direction::Down) => (w_len, w_len + 2),
                            (2, Direction::Right) => ((w_o.position.x - w.position.x + 2 * w_len + 1) % (2 * w_len + 2), w_o.position.x - w.position.x + 1),
                            (2, Direction::Down) => (2 * w_len + 1, 1),
                            (3, Direction::Right) => (w_len - 1, w_len + 1),
                            (3, Direction::Down) => (w_o.position.y - w.position.y, w_o.position.y - w.position.y + 2),
                            _ => unreachable!()
                        }})                       
                    }

                    WordCompatibilityError::InvalidIntersection => unreachable!(),
                }, w_o)
            })
            .collect::<Vec<(&WordCompatibilityError, (&PlacedWord<CharT, StrT>, (i16, i16)), &PlacedWord<CharT, StrT>)>>();

        log!(JsValue::from_serde(&between_word_data.iter().map(|(_, (w, a), _)| (w.position.clone(), a)).collect::<Vec<_>>()).unwrap());

        let gap: usize = 20;
        let border_radius: usize = 20;
        let cell_size: usize = 200;
        let font_size: usize = 80;
        let word_border_dist_from_cell_wall: usize = 20;
        let word_border_width: usize = 8;
        let word_border_radius: usize = 40;
        let between_word_width: usize = 8;
        let between_word_radius: usize = 20;
        let color_dark = ColorRGBA::opaque(84, 84, 84);
        let color_normal = ColorRGBA::opaque(125, 125, 125);
        let color_light = ColorRGBA::opaque(156, 156, 156);
        let color_error_dark = ColorRGBA::opaque(255, 84, 84);
        let color_error_normal = ColorRGBA::opaque(255, 125, 125);
        let color_error_light = ColorRGBA::opaque(255, 156, 156);

        let cell_html = cell_data.iter().map(|(pos, (words_and_indexes, compatibility_errors))|
        {
            let characters = words_and_indexes.iter().map(|(w, i)| w.value.as_ref()[*i].clone()).collect::<HashSet<_>>();
            let character = (characters.len() == 1).then_some(characters.into_iter().next().unwrap());

            let words_visible_when_hovered = words_and_indexes.iter().map(|(w, _)| 
                css!(
                    :hover ~ #${format!("word{}", word_data.get(w).unwrap().0)} 
                    { 
                        visibility: visible;
                    }
            )).collect::<Vec<_>>();

            let (background_color, hover_background_color) = 
                if let Some(_) = &character 
                    { (color_light, color_normal) } 
                else 
                    { (color_error_light, color_error_normal) };

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

                        :hover
                        {  
                            background-color: ${hover_background_color};
                        }
                    ),
                    words_visible_when_hovered,
                    css!(
                        left: ${pos.x as isize * cell_size as isize + pos.x as isize * gap as isize}px;
                        top: ${pos.y as isize * cell_size as isize + pos.y as isize * gap as isize}px;
                    )
                )}>
                    { character }
                </div>
            }

        });

        let between_cell_html = between_cell_data.iter().map(|((pos, dir), (words, compatibility_errors))|
        {
            let (width, height) = match dir
            {
                Direction::Right => (gap, cell_size),
                Direction::Down => (cell_size, gap),
            };

            let styles = words.iter().map(|w| 
                css!(
                    :hover ~ #${format!("word{}", word_data.get(w).unwrap().0)} 
                    { 
                        visibility: visible;
                    }
            )).collect::<Vec<_>>();

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
                    styles,
                    css!(
                        left: ${(pos.x as isize + (*dir == Direction::Right) as isize) * cell_size as isize + pos.x as isize * gap as isize}px;
                        top: ${(pos.y as isize + (*dir == Direction::Down) as isize) * cell_size as isize + pos.y as isize * gap as isize}px;
                    )
                )}/>
            }
        });


        let word_html = word_data.iter().map(|(w, (i, errors))| 
        {
            let (width, height) = match &w.direction
            {
                Direction::Right => (w.value.as_ref().len(), 1),
                Direction::Down => (1, w.value.as_ref().len()),
            };

            let word_red_when_errors = (!errors.is_empty()).then_some(
                css!(
                    visibility: visible;
                    border-color: ${color_error_dark};
            ));

            html!
            {
                <div id={format!("word{}", i)} 
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
                    css!
                    (
                        left: ${w.position.x as isize * (cell_size + gap) as isize + word_border_dist_from_cell_wall as isize}px;
                        top: ${w.position.y as isize * (cell_size + gap) as isize + word_border_dist_from_cell_wall as isize}px;
                        width: ${width * (cell_size + gap) - gap - 2 * word_border_dist_from_cell_wall}px;
                        height: ${height * (cell_size + gap) - gap - 2 * word_border_dist_from_cell_wall}px;
                    )
                )}/>
            }
        });

        let between_word_html = between_word_data.iter().flat_map(|(compatibility_error, (w, (start, end)), w2)|
        {
            let start =  *start as usize;
            let end = *end as usize;
            let w_len = w.value.as_ref().len() as usize;
            let (w_width, w_height) = match &w.direction
            {
                Direction::Right => (w_len, 1),
                Direction::Down => (1, w_len),
            };
            
            match &w.direction
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
                        0 => ((w.position.x as isize + (curr - side_start) as isize) * (cell_size + gap) as isize - gap as isize / 2 + div_start as isize, w.position.y as isize * (cell_size + gap) as isize - gap as isize / 2 + between_word_width as isize / 2),
                        1 => ((w.position.x as isize + side_start as isize) * (cell_size + gap) as isize - gap as isize / 2 - between_word_width as isize / 2, (w.position.y as isize + (curr - side_start) as isize) * (cell_size + gap) as isize - gap as isize / 2 + div_start as isize),
                        2 => ((w.position.x as isize + w_width as isize - (curr - side_start) as isize) * (cell_size + gap) as isize - gap as isize / 2 - div_start as isize, (w.position.y as isize + w_height as isize) * (cell_size + gap) as isize - gap as isize / 2 - between_word_width as isize / 2),
                        3 => (w.position.x as isize * (cell_size + gap) as isize - gap as isize / 2 + between_word_width as isize / 2, (w.position.y as isize + w_height as isize - (curr - side_start) as isize) * (cell_size + gap) as isize - gap as isize / 2 - div_start as isize),
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

                    let (x, y) = (x as isize + w.position.x as isize, y as isize + w.position.y as isize);
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
            })
        });

        self.html = cell_html.chain(between_cell_html).chain(word_html).chain(between_word_html).collect();

    }
}

impl<CharT: CrosswordChar + ToHtml + 'static, StrT: CrosswordString<CharT> + 'static> Component for PlaygroundComponent<CharT, StrT>
{
    type Properties = PlaygroundComponentProps<CharT, StrT>;
    type Message = PlaygroundComponentMessage<CharT, StrT>;

    fn create(ctx: &Context<Self>) -> Self
    {
        ctx.props().link.borrow_mut().replace(ctx.link().clone());
        let mut this = PlaygroundComponent
        {
            words: ctx.props().words.clone(),
            word_compatibility_settings: ctx.props().word_compatibility_settings.clone(),
            transform_x: 0f32,
            transform_y: 0f32,
            transform_zoom: 0.3f32,

            html: Vec::default(),
        };
        this.recalculate_drawing_data();
        this
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool 
    {
        match msg
        {
            PlaygroundComponentMessage::SetWords(ws) => { self.words = ws; self.recalculate_drawing_data(); },
            PlaygroundComponentMessage::SetCrossword(cw) => { self.words = cw.into_iter().collect(); self.recalculate_drawing_data(); },
            PlaygroundComponentMessage::AddWord(w) => { self.words.push(w); self.recalculate_drawing_data(); },
            PlaygroundComponentMessage::RemoveWord(s) => if let Some(pos) = self.words.iter().position(|x| x.value == s) { self.words.remove(pos); self.recalculate_drawing_data(); }
            PlaygroundComponentMessage::Scroll(amount_x, amount_y) => { self.transform_x += amount_x; self.transform_y += amount_y; },
            PlaygroundComponentMessage::Zoom(amount) => self.transform_zoom *= amount,
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool 
    {
        if ctx.props().words != _old_props.words
        {
            ctx.link().send_message(PlaygroundComponentMessage::SetWords(ctx.props().words.clone()));
            return false;
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html
    {

        html! 
        {
            <div class={classes!("playground-wrapper",
                css!
                (
                    width: 100%;
                    height: 100%;
                    overflow: hidden;
                    position: relative;
                    user-select: none;
                )
            )} 
            onwheel=
            {
                let ctx_link = ctx.link().clone();
                Callback::from(move |event: WheelEvent| 
                {
                    event.stop_propagation();
                    event.prevent_default();

                    let curr_el = event.current_target().unwrap().dyn_into::<HtmlElement>().unwrap();
                    let child = curr_el.get_elements_by_class_name("playground").get_with_index(0).unwrap().dyn_into::<HtmlElement>().unwrap();
                    let child_bounding_box = child.get_bounding_client_rect();

                    let zoom = 1.001f32.powf(-event.delta_y() as f32);
                    ctx_link.send_message(PlaygroundComponentMessage::Scroll((event.client_x() as f32 - child_bounding_box.left() as f32) * (1f32 - zoom), (event.client_y() as f32 - child_bounding_box.top() as f32) * (1f32 - zoom)));
                    ctx_link.send_message(PlaygroundComponentMessage::Zoom(zoom));
                })
            }
            onmousemove=
            {
                let ctx_link = ctx.link().clone();
                Callback::from(move |event: MouseEvent| 
                {
                    event.stop_propagation();
                    if event.buttons() == 1
                    {
                        ctx_link.send_message(PlaygroundComponentMessage::Scroll(event.movement_x() as f32, event.movement_y() as f32)); 
                    }
                })
            }>
                <div class={classes!("playground", 
                    css!
                    (
                        width: 100%;
                        height: 100%;
                        position: relative;
                        user-select: none;
                        transform-origin: 0 0;
                    )
                )} style={format!("transform: translate({}px, {}px) scale({})", self.transform_x, self.transform_y, self.transform_zoom)}>

                { for self.html.iter().cloned() }

                

                </div>
            </div>
        }
    }
}
