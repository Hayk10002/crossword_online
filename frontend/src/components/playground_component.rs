#![allow(non_upper_case_globals)]
use std::cell::Cell;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::default;
use std::env::current_exe;
use std::io::Empty;
use std::iter::{empty, once, repeat};
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use std::thread::current;

use _PlaygroundComponentProps::word_compatibility_settings;
use crossword_generator::crossword::{Crossword, CrosswordError, WordCompatibilityError, WordCompatibilitySettings};
use crossword_generator::placed_word::PlacedWord;
use crossword_generator::traits::{CrosswordChar, CrosswordString};
use crossword_generator::word::{Direction, Position};
use gloo_console::log;
use html::{IntoPropValue, Scope};
use itertools::Itertools;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use web_sys::js_sys::Array;
use yew::prelude::*;
use stylist::{css, style};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use gloo_utils::format::JsValueSerdeExt;
use gloo_utils::document;
use gloo_timers::callback::Timeout;
use web_sys::{DataTransfer, Element, HtmlElement};
use web_sys::CssStyleDeclaration;
use yew::virtual_dom::VNode;
use web_sys::WheelEvent;

use crate::components::playground_children_components::{PlaygroundBetweenCellComponent, PlaygroundCellComponent, PlaygroundWordComponent};
use crate::utils::color_rgba::ColorRGBA;
use crate::utils::settings::StyleSettings;

use super::super::utils::weak_component_link::WeakComponentLink;
use super::playground_children_components::PlaygroundWordErrorOutlineComponent;

#[derive(Default, Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Hash)]
pub enum PlaygroundWordState
{
    #[default]
    Normal,
    Selected,
    Phantom,
}

#[derive(Default, Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Hash)]
pub struct PlaygroundWord<CharT: CrosswordChar, StrT: CrosswordString<CharT>>
{
    pub w: PlacedWord<CharT, StrT>,
    pub state: PlaygroundWordState,
}

impl<CharT: CrosswordChar, StrT: CrosswordString<CharT>> PlaygroundWord<CharT, StrT>
{
    fn new(w: PlacedWord<CharT, StrT>, state: PlaygroundWordState) -> PlaygroundWord<CharT, StrT>
    {
        PlaygroundWord { w, state }
    }

    fn from_placed_word(w: PlacedWord<CharT, StrT>) -> PlaygroundWord<CharT, StrT>
    {
        PlaygroundWord::new(w, PlaygroundWordState::Normal)
    }
}

impl<CharT: CrosswordChar, StrT: CrosswordString<CharT>> Deref for PlaygroundWord<CharT, StrT>
{
    type Target = PlacedWord<CharT, StrT>;

    fn deref(&self) -> &Self::Target
    {
        &self.w
    }
}

impl<CharT: CrosswordChar, StrT: CrosswordString<CharT>> DerefMut for PlaygroundWord<CharT, StrT>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.w
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize)]
pub struct PlaygroundWordId
{
    id: usize,
}

impl PlaygroundWordId
{
    fn id(&self) -> usize { self.id }
    fn counter() -> &'static Mutex<usize>
    {
        static COUNTER: Mutex<usize> = Mutex::new(0);
        &COUNTER
    }
    fn new() -> PlaygroundWordId 
    {
        let mut counter = PlaygroundWordId::counter().lock().unwrap();
        let new_id = *counter;
        *counter += 1;
        PlaygroundWordId{ id: new_id }
    }
}

#[derive(PartialEq, Properties)]
pub struct PlaygroundComponentProps<CharT, StrT>
where 
    CharT: CrosswordChar + ToHtml + 'static,
    StrT: CrosswordString<CharT> + 'static,
    CharT: Serialize + DeserializeOwned, 
    StrT: Serialize + DeserializeOwned,
{
    pub words: Vec<PlacedWord<CharT, StrT>>,
    pub word_compatibility_settings: WordCompatibilitySettings,
    pub link: WeakComponentLink<PlaygroundComponent<CharT, StrT>>
}

#[derive(Debug, Clone)]
pub enum PlaygroundComponentMessage<CharT: CrosswordChar, StrT: CrosswordString<CharT>>
{
    SetCrossword(Crossword<CharT, StrT>),
    SetWords(Vec<PlacedWord<CharT, StrT>>),
    AddWord(PlaygroundWord<CharT, StrT>),
    RemoveWord(PlaygroundWordId),
    ChangeWord(PlaygroundWordId, PlaygroundWord<CharT, StrT>),
    SelectWord(PlaygroundWordId),
    SelectAll,
    DeselectWord(PlaygroundWordId),
    DeselectAll,
    RemoveAllPhantoms,
    SetDragging(bool),
    StartDragging(WordsType<CharT, StrT>),
    DropDragging,
    EndDragging,

    MessageBatch(Vec<PlaygroundComponentMessage<CharT, StrT>>),

    Scroll(f32, f32),
    Zoom(f32),

    SetDraggingOffset(f32, f32),
    SetDraggingDivPos(f32, f32),
}


type WordsType<CharT, StrT> = HashMap<PlaygroundWordId, PlaygroundWord<CharT, StrT>>;
type WordDataType = HashMap<PlaygroundWordId, Vec<(WordCompatibilityError, PlaygroundWordId)>>;
type CellDataType = HashMap<Position, (Vec<(PlaygroundWordId, usize)>, Vec<(WordCompatibilityError, PlaygroundWordId)>)>;
type BetweenCellDataType = HashMap<(Position, Direction), (Vec<PlaygroundWordId>, Vec<(WordCompatibilityError, PlaygroundWordId)>)>;
type BetweenWordDataType = Vec<(WordCompatibilityError, (PlaygroundWordId, (i16, i16)), PlaygroundWordId)>;

#[derive(Clone)]
pub struct PlaygroundComponent<CharT: CrosswordChar, StrT: CrosswordString<CharT>>
{
    words: WordsType<CharT, StrT>,
    word_compatibility_settings: WordCompatibilitySettings,

    dragging_words: WordsType<CharT, StrT>,
    dragging_mouse_offset_x: f32, 
    dragging_mouse_offset_y: f32, 

    dragging_div_pos_x: f32, 
    dragging_div_pos_y: f32, 

    currently_dragging: bool,
    
    transform_x: f32,
    transform_y: f32,
    transform_zoom: f32,

    html: Vec<VNode>,
    dragging_html: Vec<VNode>,

    playground_node_ref: NodeRef,
    dragging_node_ref: NodeRef,
    dragging_image_ref: NodeRef,
}

impl<CharT, StrT> PlaygroundComponent<CharT, StrT>
    where 
        CharT: CrosswordChar + ToHtml + 'static,
        StrT: CrosswordString<CharT> + 'static,
        CharT: Serialize + DeserializeOwned,
        StrT: Serialize + DeserializeOwned,
{
    
    fn apply_message(&mut self, ctx: &Context<Self>, msg: PlaygroundComponentMessage<CharT, StrT>) -> bool
    {
        let msg_clone = msg.clone();
        log!(format!("Message {:?}", msg_clone));
        match msg
        {
            PlaygroundComponentMessage::SetWords(ws) => 
            {
                self.words = ws.into_iter().map(|x| (PlaygroundWordId::new(), PlaygroundWord::from_placed_word(x))).collect(); 
                true
            },
            PlaygroundComponentMessage::SetCrossword(cw) => 
            { 
                self.words = cw.into_iter().map(|x| (PlaygroundWordId::new(), PlaygroundWord::from_placed_word(x))).collect(); 
                true 
            },
            PlaygroundComponentMessage::AddWord(w) => 
            { 
                self.words.insert(PlaygroundWordId::new(), w); 
                true
            },
            PlaygroundComponentMessage::RemoveWord(w_id) => if let Some(_) = self.words.remove(&w_id) { true } else { false }
            PlaygroundComponentMessage::ChangeWord(w_id, other) => 
                if let Some(w) = self.words.get_mut(&w_id)
                {
                    if *w != other
                    { 
                        *w = other;
                        true
                    } else { false }
                } else { false }
            PlaygroundComponentMessage::SelectWord(w_id) =>
                if let Some(w) = self.words.get_mut(&w_id)
                {
                    if w.state == PlaygroundWordState::Normal
                    { 
                        w.state = PlaygroundWordState::Selected;
                        true
                    } else { false }
                } else { false }
            PlaygroundComponentMessage::SelectAll =>
            {
                self.words.iter_mut().filter(|(_, w)| w.state == PlaygroundWordState::Normal).for_each(|(_, w)| w.state = PlaygroundWordState::Selected);
                true
            }
            PlaygroundComponentMessage::DeselectWord(w_id) => 
                if let Some(w) = self.words.get_mut(&w_id)
                {
                    if w.state == PlaygroundWordState::Selected
                    { 
                        w.state = PlaygroundWordState::Normal;
                        true
                    } else { false }
                } else { false }
            PlaygroundComponentMessage::DeselectAll =>
            {
                self.words.iter_mut().filter(|(_, w)| w.state == PlaygroundWordState::Selected).for_each(|(_, w)| w.state = PlaygroundWordState::Normal);
                true
            }
            PlaygroundComponentMessage::RemoveAllPhantoms =>
            {
                let to_remove = self.words.iter().filter_map(|(id, w)| (w.state == PlaygroundWordState::Phantom).then_some(*id)).collect_vec();
                to_remove.into_iter().for_each(|id| { self.words.remove(&id); });
                true
            }
            
            PlaygroundComponentMessage::SetDragging(val) =>
            {
                self.currently_dragging = val;
                true
            }
            PlaygroundComponentMessage::StartDragging(words) => 
            {
                self.dragging_words = words;
                for (id, w) in self.dragging_words.iter()
                {
                    self.words.insert(*id, PlaygroundWord::new(w.w.clone(), PlaygroundWordState::Phantom));   
                }
                true
            }
            PlaygroundComponentMessage::DropDragging =>
            {
                let words_to_insert = self.dragging_words.keys().filter_map(|id| self.words.get(id)).map(|w| PlaygroundWord::new(w.w.clone(), PlaygroundWordState::Normal)).collect_vec();

                words_to_insert.into_iter().for_each(|w| { self.words.insert(PlaygroundWordId::new(), w); });
                
                true
            }
            PlaygroundComponentMessage::EndDragging =>
            {
                for id in self.dragging_words.keys()
                {
                    self.words.remove(id);
                }
                self.dragging_words.clear();
                true
            }
            
            PlaygroundComponentMessage::MessageBatch(messages) => 
            {
                let this = self.clone();
                messages.into_iter().for_each(|msg| { self.apply_message(ctx, msg); });

                this.words != self.words || this.dragging_words != self.dragging_words
            },
            
            PlaygroundComponentMessage::Scroll(amount_x, amount_y) => { self.transform_x += amount_x; self.transform_y += amount_y; false },
            PlaygroundComponentMessage::Zoom(amount) => { self.transform_zoom *= amount; false },

            PlaygroundComponentMessage::SetDraggingOffset(x, y) => { self.dragging_mouse_offset_x = x; self.dragging_mouse_offset_y = y; false }
            PlaygroundComponentMessage::SetDraggingDivPos(x, y) => 
            { 
                self.dragging_div_pos_x = x; 
                self.dragging_div_pos_y = y; 

                let set = ctx.link().context::<StyleSettings>(Callback::noop()).unwrap().0;
                let cell_size = set.playground_style_settings.cell_size as f32;
                let gap = set.playground_style_settings.gap as f32;

                let ph_off_x = ((x + (cell_size + gap) / 2.0) / (cell_size + gap)).floor() as isize;
                let ph_off_y = ((y + (cell_size + gap) / 2.0) / (cell_size + gap)).floor() as isize;

                let mut changed = false;

                for (id, w) in self.dragging_words.iter()
                {
                    let pos = w.position.clone();
                    let w = self.words.get_mut(id).unwrap();
                    let new_pos = Position { x: pos.x + ph_off_x as i16, y: pos.y + ph_off_y as i16 }; 
                    if w.position != new_pos
                    {
                        w.position = new_pos;
                        changed = true;
                    }
                }
                changed 
            }
        }
    }


    fn calculate_word_data(words: &WordsType<CharT, StrT>, word_comp_settings: &WordCompatibilitySettings) -> WordDataType
    {
        let mut word_data = words.keys().map(|i| (*i, (Vec::<(WordCompatibilityError, PlaygroundWordId)>::default()))).collect::<HashMap<_, _>>();
        for comb in words.keys().combinations(2)
        {
            if let Some(error) = word_comp_settings.word_compatibility_issue(&words[comb[0]], &words[comb[1]])
            {
                word_data.get_mut(comb[0]).unwrap().push((error.clone(), *comb[1]));
                word_data.get_mut(comb[1]).unwrap().push((error.clone(), *comb[0]));
            }           
        };
        word_data
    }

    fn calculate_cell_data(word_data: &WordDataType, words: &WordsType<CharT, StrT>) -> CellDataType
    {
        let mut cell_data: CellDataType = HashMap::new();

        for w in word_data.iter().filter_map(|(w, _)| words.get(w))
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

        for (&w_id, w, errors) in word_data.iter().filter_map(|(w_id, e)| words.get(w_id).map(|w| (w_id, w, e)))
        {
            for i in 0..w.value.as_ref().len()
            {
                let cell = match &w.direction
                {
                    Direction::Right => cell_data.get_mut(&Position { x: w.position.x + i as i16, y: w.position.y }), 
                    Direction::Down => cell_data.get_mut(&Position { x: w.position.x, y: w.position.y + i as i16}), 
                }.unwrap();
                cell.0.push((w_id, i));
                cell.1.extend(errors.iter().cloned());
            }
        }

        cell_data
    }

    fn calculate_between_cell_data(word_data: &WordDataType, words: &WordsType<CharT, StrT>) -> BetweenCellDataType
    {
        let mut between_cell_data: BetweenCellDataType = HashMap::new();

        for w in word_data.iter().filter_map(|(w, _)| words.get(w))
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

        for (&w_id, w, errors) in word_data.iter().filter_map(|(w_id, e)| words.get(w_id).map(|w| (w_id, w, e)))
        {
            for i in 0..w.value.as_ref().len() - 1
            {
                let between_cell = match &w.direction
                {
                    Direction::Right => between_cell_data.get_mut(&(Position { x: w.position.x + i as i16, y: w.position.y }, Direction::Right)), 
                    Direction::Down => between_cell_data.get_mut(&(Position { x: w.position.x, y: w.position.y + i as i16}, Direction::Down)), 
                }.unwrap();
                between_cell.0.push(w_id);
                between_cell.1.extend(errors.iter().cloned());
            }
        }

        between_cell_data
    }

    fn calculate_between_word_data(word_data: &WordDataType, words: &WordsType<CharT, StrT>) -> BetweenWordDataType
    {
        word_data.iter()
            .flat_map(|(&w, errors)| 
                repeat(w)
                .zip(errors.iter())
                .map(|(w1, (e, w2))| (e, w1, *w2))
                .filter(|(e, _, _)| if let &&WordCompatibilityError::InvalidIntersection = e { false } else { true }))
            .map(|(e, w_id, w_o_id)| 
            {
                let w = words.get(&w_id).unwrap();
                let w_o = words.get(&w_o_id).unwrap();
                let w_len = w.value.as_ref().len() as i16; 
                (e.clone(), (w_id, match e
                {
                    WordCompatibilityError::CornerByCorner => 
                    {
                        let w_corner = (w_o.position.x < w.position.x) as usize * 2 + (w_o.position.y < w.position.y) as usize;// 0 leftup, 1 leftdown, 2 rightup, 3 rightdown

                        match w_corner
                        {
                            0 => (w_len, w_len + 1),
                            1 => if w.direction == Direction::Down { (0, 1) } else { (w_len - 1, w_len) },
                            2 => if w.direction == Direction::Down { (w_len + 1, w_len + 2) } else { (2 * w_len, 2 * w_len + 1) },
                            3 => (2 * w_len + 1, 0),
                            _ => unreachable!()
                        }
                    },

                    WordCompatibilityError::HeadByHead =>
                    {
                        let w_side = ((w.direction == Direction::Down) as usize * 2 + (w_o.position.x + w_o.position.y < w.position.x + w.position.y) as usize + 3) % 4; // 0 right, 1 up, 2 down, 3 left

                        match w_side
                        {
                            0 => (2 * w_len, 0),
                            1 => (w_len, w_len + 2),
                            2 => (2 * w_len + 1, 1),
                            3 => (w_len - 1, w_len + 1),
                            _ => unreachable!()
                        }
                    },

                    WordCompatibilityError::SideBySide =>
                    {
                        let dir = (w.direction == Direction::Right) as usize;
                        let side_com = if dir == 1 { w_o.position.y < w.position.y } else { w_o.position.x < w.position.x } as usize;
                        let w_side = (dir * 2 + side_com + 3) % 4; // 0 right, 1 up, 2 down, 3 left

                        let w_o_len = w_o.value.as_ref().len() as i16; 
                        match w_side
                        {
                            0 => (max(w_len + 1, (w.position.y + w_len) - (w_o.position.y + w_o_len) + w_len + 1), min(w.position.y - w_o.position.y + 2 * w_len + 2, 2 * w_len + 2) % (2 * w_len + 2)),
                            1 => (max(w_len, (w.position.x + w_len) - (w_o.position.x + w_o_len) + w_len), min(w.position.x - w_o.position.x + 2 * w_len + 1, 2 * w_len + 1)),
                            2 => ((max(-1, w_o.position.x - w.position.x - 1) + 2 * w_len + 2) % (2 * w_len + 2), min((w_o.position.x + w_o_len) - (w.position.x + w_len) + w_len, w_len)),
                            3 => (max(0, w_o.position.y - w.position.y), min((w_o.position.y + w_o_len) - (w.position.y + w_len) + w_len as i16 + 1, w_len as i16 + 1)),
                            _ => unreachable!()
                        }
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

                        match (w_side, &w.direction)
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
                        }                       
                    }

                    WordCompatibilityError::InvalidIntersection => unreachable!(),
                }), w_o_id)
            })
            .collect()
    }

    fn generate_cell_html(cell_data: &CellDataType, words: &WordsType<CharT, StrT>, ctx: &Context<Self>) -> Vec<VNode>
    {
        cell_data.iter().map(|(pos, (words_and_indexes, compatibility_errors))|
        {
            let characters = words_and_indexes.iter().map(|(w_id, i)| words[w_id].value.as_ref()[*i].clone()).collect::<HashSet<_>>();
            let character = (characters.len() == 1).then_some(characters.into_iter().next().unwrap());
            let word_ids = words_and_indexes.iter().map(|(w_id, _)| w_id.id()).collect_vec();
            let phantom = words_and_indexes.iter().all(|(w_id, _)| words[w_id].state == PlaygroundWordState::Phantom);
            let selected = words_and_indexes.iter().any(|(w_id, _)| words[w_id].state == PlaygroundWordState::Selected);

            let state = if phantom { PlaygroundWordState::Phantom } else if selected { PlaygroundWordState::Selected } else { PlaygroundWordState::Normal };


            html!
            {
                <PlaygroundCellComponent<CharT> character={character} word_ids={word_ids} position={pos.clone()} state={state} 
                    // on_invert_word_direction=
                    // {
                    //     let ctx_link = ctx.link().clone();
                    //     let words_with_changes = words_and_indexes.iter().map(|&(w, i)| 
                    //     {
                    //         let mut changed_word = w.clone();
                    //         changed_word.position = match &changed_word.direction
                    //         {
                    //             Direction::Right => Position { x: changed_word.position.x + i as i16, y: changed_word.position.y - i as i16},
                    //             Direction::Down => Position { x: changed_word.position.x - i as i16, y: changed_word.position.y + i as i16} 
                    //         };
                    //         changed_word.direction = changed_word.direction.opposite();
        
                    //         (w.clone(), changed_word)
                    //     }).collect_vec();
                    //     Callback::from(move |_| ctx_link.send_message(PlaygroundComponentMessage::MessageBatch(words_with_changes.iter().map(|(w, ch_w)| PlaygroundComponentMessage::ChangeWord(w.clone(), ch_w.clone())).collect_vec())))
                    // }
                    on_select=
                    { 
                        let ctx_link = ctx.link().clone();
                        let w_id = words_and_indexes.iter().skip_while(|(w_id, _)| words[w_id].state == PlaygroundWordState::Selected).next().map(|(w_id, _)| w_id);
                        let messages = match w_id
                        {
                            Some(id) => once(PlaygroundComponentMessage::SelectWord(*id)).collect_vec(),
                            None => words_and_indexes.iter().map(|(w_id, _)| PlaygroundComponentMessage::DeselectWord(*w_id)).collect_vec(),
                        };
                        Callback::from(move |ctrl_pressed: bool| ctx_link.send_message(PlaygroundComponentMessage::MessageBatch((!ctrl_pressed).then_some(PlaygroundComponentMessage::DeselectAll).into_iter().chain(messages.iter().cloned()).collect_vec())))
                    }
                />
            }
        }).collect_vec()
    }

    fn generate_between_cell_html(between_cell_data: &BetweenCellDataType, words: &WordsType<CharT, StrT>, ctx: &Context<Self>) -> Vec<VNode>
    {
        between_cell_data.iter().map(|((pos, dir), (word_ids, compatibility_errors))|
        {
            let word_ids_nums = word_ids.iter().map(|w_id| w_id.id()).collect_vec();
            let draggable = word_ids.iter().any(|id| words[id].state == PlaygroundWordState::Selected);
            html!
            {
                <PlaygroundBetweenCellComponent position={pos.clone()} direction={dir.clone()} word_ids={word_ids_nums} draggable={draggable}
                    on_select=
                    { 
                        let ctx_link = ctx.link().clone();
                        let w_id = word_ids.iter().skip_while(|w_id| words[*w_id].state == PlaygroundWordState::Selected).next();
                        let messages = match w_id
                        {
                            Some(id) => once(PlaygroundComponentMessage::SelectWord(*id)).collect_vec(),
                            None => word_ids.iter().map(|w_id| PlaygroundComponentMessage::DeselectWord(*w_id)).collect_vec(),
                        };
                        Callback::from(move |ctrl_pressed: bool| ctx_link.send_message(PlaygroundComponentMessage::MessageBatch((!ctrl_pressed).then_some(PlaygroundComponentMessage::DeselectAll).into_iter().chain(messages.iter().cloned()).collect_vec())))
                    }
                />
            }
        }).collect_vec()
    }

    fn generate_word_html(word_data: &WordDataType, words: &WordsType<CharT, StrT>) -> Vec<VNode>
    {
        word_data.iter().map(|(w_id, errors)| 
        {
            let w = &words[w_id];
            let (width, height) = match &w.direction
            {
                Direction::Right => (w.value.as_ref().len(), 1),
                Direction::Down => (1, w.value.as_ref().len()),
            };

            let other_words_phantom = !errors.is_empty() && errors.iter().all(|(_, other_w_id)| words[other_w_id].state == PlaygroundWordState::Phantom);
            let mut state = w.state.clone();
            if state == PlaygroundWordState::Normal && other_words_phantom { state = PlaygroundWordState::Phantom; }

            html!
            {
                <PlaygroundWordComponent position={w.position.clone()} width={width} height={height} id={w_id.id()} error_exists={!errors.is_empty()} state={state}/>
            }
        }).collect_vec()
    }

    fn generate_between_word_html(between_word_data: &BetweenWordDataType, words: &WordsType<CharT, StrT>) -> Vec<VNode>
    {
        between_word_data.iter().map(|(compatibility_error, (w_id, (start, end)), w2_id)|
        {
            let w = &words[w_id];
            let w2 = &words[w2_id];
            html! { <PlaygroundWordErrorOutlineComponent position={w.position.clone()} direction={w.direction.clone()} length={w.value.as_ref().len()} start={*start as usize} end={*end as usize} phantom={w.state == PlaygroundWordState::Phantom || w2.state == PlaygroundWordState::Phantom}/> }
        }).collect_vec()
    }

    fn recalculate_main_drawing_data(&mut self, ctx: &Context<Self>)
    {
        let word_data = PlaygroundComponent::calculate_word_data(&self.words, &self.word_compatibility_settings);

        let cell_data = PlaygroundComponent::calculate_cell_data(&word_data, &self.words);

        let between_cell_data = PlaygroundComponent::calculate_between_cell_data(&word_data, &self.words);
        
        let between_word_data = PlaygroundComponent::calculate_between_word_data(&word_data, &self.words);



        let cell_html = PlaygroundComponent::generate_cell_html(&cell_data, &self.words, ctx).into_iter();

        let between_cell_html = PlaygroundComponent::generate_between_cell_html(&between_cell_data, &self.words, ctx).into_iter();

        let word_html = PlaygroundComponent::generate_word_html(&word_data, &self.words).into_iter();

        let between_word_html = PlaygroundComponent::generate_between_word_html(&between_word_data, &self.words).into_iter();

        self.html = cell_html.chain(between_cell_html).chain(word_html).chain(between_word_html).collect();

    }

    fn recalculate_dragging_drawing_data(&mut self, ctx: &Context<Self>)
    {
        let word_data = PlaygroundComponent::calculate_word_data(&self.dragging_words, &self.word_compatibility_settings);

        let cell_data = PlaygroundComponent::calculate_cell_data(&word_data, &self.dragging_words);

        let cell_html = PlaygroundComponent::generate_cell_html(&cell_data, &self.dragging_words, ctx);

        self.dragging_html = cell_html;
    }

    fn recalculate_drawing_data(&mut self, ctx: &Context<Self>)
    {
        self.recalculate_main_drawing_data(ctx);
        self.recalculate_dragging_drawing_data(ctx);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DragDataType<CharT, StrT>
where 
    CharT: CrosswordChar,
    StrT: CrosswordString<CharT>,
{
    dragging_mouse_offset_x: f32,
    dragging_mouse_offset_y: f32,
    data: Vec<(PlacedWord<CharT, StrT>, Option<PlaygroundWordId>)>
}

fn encode_uppercase(s: &str) -> String
{
    s.chars()
        .chunk_by(|ch| ch.is_ascii_uppercase()).into_iter()
        .map(|(is_upper, ch)| 
    {
        let s = ch.collect::<String>();
        if is_upper { "^[".to_owned() + &s + "]^" } else { s }
    }).fold(String::default(), |a, b| a + &b)
}

fn decode_uppercase(mut s: &str) -> String
{
    let mut current_case = false;
    let mut answer = String::default();

    while let Some(ind) = 
    {
        if !current_case 
        { 
            s.find("^[")
        }
        else
        {
            s.find("]^")
        }
    }
    {
        answer += &(if !current_case { s[..ind].to_owned() } else { s[..ind].to_ascii_uppercase() });
        current_case = !current_case;
        s = &s[ind + 2..]
    }

    if s != ""
    {
        answer += s;
    }

    answer
}

fn get_drag_data<CharT, StrT>(data_transfer: &DataTransfer) -> Option<DragDataType<CharT, StrT>>
where 
    CharT: CrosswordChar + DeserializeOwned,
    StrT: CrosswordString<CharT> + DeserializeOwned,
{
    data_transfer.types().iter()
        .map(|v| v.as_string().unwrap())
        .filter(|v| v.starts_with("application/x.word-data-"))
        .filter_map(|data| serde_json::from_str::<DragDataType<CharT, StrT>>(&decode_uppercase(&data.replace("application/x.word-data-", ""))).ok())
        .next()
}

fn set_drag_data<CharT, StrT>(data_transfer: &DataTransfer, drag_data: &DragDataType<CharT, StrT>)
where 
    CharT: CrosswordChar + Serialize,
    StrT: CrosswordString<CharT> + Serialize,
{
    data_transfer.set_data(&format!("application/x.word-data-{}", encode_uppercase(&serde_json::to_string(drag_data).unwrap())), "").unwrap();
}

impl<CharT, StrT> Component for PlaygroundComponent<CharT, StrT>
    where 
        CharT: CrosswordChar + ToHtml + 'static,
        StrT: CrosswordString<CharT> + 'static,
        CharT: Serialize + DeserializeOwned,
        StrT: Serialize + DeserializeOwned,
{
    type Properties = PlaygroundComponentProps<CharT, StrT>;
    type Message = PlaygroundComponentMessage<CharT, StrT>;

    fn create(ctx: &Context<Self>) -> Self
    {
        ctx.props().link.borrow_mut().replace(ctx.link().clone());
        let mut this = PlaygroundComponent
        {
            words: ctx.props().words.iter().cloned().map(|w| (PlaygroundWordId::new(), PlaygroundWord::from_placed_word(w))).collect(),
            word_compatibility_settings: ctx.props().word_compatibility_settings.clone(),
            transform_x: 0f32,
            transform_y: 0f32,
            transform_zoom: 0.3f32,

            dragging_words: HashMap::default(),
            dragging_mouse_offset_x: 100f32,
            dragging_mouse_offset_y: 100f32,

            dragging_div_pos_x: 0f32,
            dragging_div_pos_y: 0f32,

            currently_dragging: false,

            html: Vec::default(),
            dragging_html: Vec::default(),
            playground_node_ref: NodeRef::default(),
            dragging_node_ref: NodeRef::default(),
            dragging_image_ref: NodeRef::default(),
        };
        this.recalculate_drawing_data(ctx);
        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool 
    {
        if self.apply_message(ctx, msg)
        {
            self.recalculate_drawing_data(ctx);
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool 
    {
        if ctx.props().words != old_props.words
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
                let playground_node_ref = self.playground_node_ref.clone();
                Callback::from(move |event: WheelEvent| 
                {
                    event.stop_propagation();
                    event.prevent_default();

                    let playground_element = playground_node_ref.cast::<HtmlElement>().unwrap();
                    let playground_bounding_box = playground_element.get_bounding_client_rect();

                    let zoom = 1.001f32.powf(-event.delta_y() as f32);
                    ctx_link.send_message(PlaygroundComponentMessage::Scroll((event.client_x() as f32 - playground_bounding_box.left() as f32) * (1f32 - zoom), (event.client_y() as f32 - playground_bounding_box.top() as f32) * (1f32 - zoom)));
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
            }
            ondragstart=
            {
                let ctx_link = ctx.link().clone();
                let selected_word_ids_and_words = self.words.iter().filter_map(|(id, w)| (w.state == PlaygroundWordState::Selected).then_some((*id, w.clone()))).collect_vec();
                let playground_node_ref = self.playground_node_ref.clone();
                //let dragging_node_element = self.dragging_node_ref.cast::<Element>().unwrap();
                let dragging_image_ref = self.dragging_image_ref.clone();
                let zoom = self.transform_zoom;
                Callback::from(move |event: DragEvent| 
                {
                    //event.prevent_default();
                    let ctx_link = ctx_link.clone();
                    let words = selected_word_ids_and_words.iter().map(|(_, w)| (w.w.clone(), None)).collect_vec();
                    let ids = selected_word_ids_and_words.iter().map(|(id, _)| *id).collect_vec();

                    let playground = playground_node_ref.cast::<HtmlElement>().unwrap();
                    let playground_bounding_box = playground.get_bounding_client_rect();

                    let dragging_mouse_offset = ((event.client_x() as f32 - playground_bounding_box.left() as f32) / zoom, (event.client_y() as f32 - playground_bounding_box.top() as f32) / zoom);// = (self.dragging_mouse_offset_x, self.dragging_mouse_offset_y);
                        
                    let drag_data = DragDataType 
                    {
                        dragging_mouse_offset_x: dragging_mouse_offset.0,
                        dragging_mouse_offset_y: dragging_mouse_offset.1,
                        data: words,
                    };
                    if let Some(data_transfer) = event.data_transfer()
                    {
                        log!("dragstart");
                        data_transfer.clear_data().unwrap();
                        data_transfer.set_data("text/plain", &drag_data.data.iter().map(|(w, _)| serde_json::to_string(&w.value).unwrap()).join(",")).unwrap();
                        set_drag_data(&data_transfer, &drag_data);
                        data_transfer.set_drag_image(&dragging_image_ref.cast::<Element>().unwrap(), 0, 0);
                    }

                    Timeout::new(1, move || 
                    {
                        ctx_link.send_message(PlaygroundComponentMessage::MessageBatch(once(PlaygroundComponentMessage::SetDragging(true)).chain(ids.into_iter().map(|id| PlaygroundComponentMessage::RemoveWord(id))).collect_vec()));
                    }).forget();
                })
            }
            // ondrag=
            // {
                
            // }
            ondragend=
            {
                let ctx_link = ctx.link().clone();
                Callback::from(move |event: DragEvent|
                {
                    log!("end");
                    if let Some(data_transfer) = event.data_transfer()
                    {
                        data_transfer.clear_data().unwrap();
                    }
                    ctx_link.send_message(PlaygroundComponentMessage::MessageBatch(vec![PlaygroundComponentMessage::SetDragging(false), PlaygroundComponentMessage::EndDragging]));
                })
            }
            ondragenter=
            {
                let ctx_link = ctx.link().clone();
                Callback::from(move |event: DragEvent|
                {
                    log!("dragenter");
                    if let Some(data_transfer) = event.data_transfer()
                    {
                        if let Some(mut drag_data) = get_drag_data(&data_transfer)
                        {
                            drag_data.data.iter_mut().for_each(|(_, id)| *id = Some(PlaygroundWordId::new()));
                            set_drag_data(&data_transfer, &drag_data);

                            let ctx_link = ctx_link.clone();

                            ctx_link.send_message(PlaygroundComponentMessage::SetDraggingOffset(drag_data.dragging_mouse_offset_x, drag_data.dragging_mouse_offset_y));
                            ctx_link.send_message(PlaygroundComponentMessage::MessageBatch(vec![PlaygroundComponentMessage::SetDragging(true), PlaygroundComponentMessage::StartDragging(drag_data.data.into_iter().map(|(w, id)| (id.unwrap(), PlaygroundWord::from_placed_word(w))).collect())]));
                        }
                    }
                })
            }
            ondragover=
            {
                let ctx_link = ctx.link().clone();
                let playground_node_ref = self.playground_node_ref.clone();
                let zoom = self.transform_zoom;
                let dragging_mouse_offset_x = self.dragging_mouse_offset_x;
                let dragging_mouse_offset_y = self.dragging_mouse_offset_y;
                Callback::from(move |event: DragEvent|
                {
                    event.prevent_default();
                    let playground = playground_node_ref.cast::<HtmlElement>().unwrap();
                    let playground_bounding_box = playground.get_bounding_client_rect();

                    let x = (event.client_x() as f32 - playground_bounding_box.left() as f32) / zoom - dragging_mouse_offset_x;
                    let y = (event.client_y() as f32 - playground_bounding_box.top() as f32) / zoom - dragging_mouse_offset_y;
                    ctx_link.send_message(PlaygroundComponentMessage::SetDraggingDivPos(x, y));
                })
            }
            ondragleave=
            {
                let ctx_link = ctx.link().clone();
                Callback::from(move |event: DragEvent|
                {
                    ctx_link.send_message(PlaygroundComponentMessage::MessageBatch(vec![PlaygroundComponentMessage::EndDragging]));
                    log!("dragleave");
                })
            }
            ondrop=
            {
                let ctx_link = ctx.link().clone();
                Callback::from(move |event: DragEvent|
                {
                    ctx_link.send_message(PlaygroundComponentMessage::MessageBatch(vec![PlaygroundComponentMessage::DropDragging, PlaygroundComponentMessage::SetDragging(false), PlaygroundComponentMessage::EndDragging]));
                })
            }>
                <div ref={ self.playground_node_ref.clone() } class={classes!("playground", 
                    css!
                    (
                        position: relative;
                        user-select: none;
                        transform-origin: 0 0;
                    ),
                    self.currently_dragging.then_some(css!
                    (
                        * 
                        {
                            pointer-events: none;
                        }
                    ))
                )} style={format!("transform: translate({}px, {}px) scale({})", self.transform_x, self.transform_y, self.transform_zoom)}>

                { for self.html.iter().cloned() }

                    <div ref={ self.dragging_node_ref.clone() } class={classes!("dragging-image", 
                        css!
                        (
                            position: relative;
                            user-select: none;
                            transform-origin: 0 0;
                        )
                    )} style={format!("transform: translate({}px, {}px)", self.dragging_div_pos_x, self.dragging_div_pos_y)}>
                    
                    { for self.dragging_html.iter().cloned() }
                    
                    </div>
                    <div ref={ self.dragging_image_ref.clone() } class={classes!("dragging-ghost", 
                        css!
                        (
                            position: absolute;
                            width: 0;
                            height: 0;
                        )
                    )}/> 

                </div>
            </div>
        }
    }
}
