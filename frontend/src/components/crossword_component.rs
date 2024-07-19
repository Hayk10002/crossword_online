#![allow(non_upper_case_globals)]
use std::iter::repeat;

use crossword_generator::crossword::Crossword;
use crossword_generator::placed_word::PlacedWord;
use gloo_console::log;
use yew::prelude::*;
use stylist::css;

use super::super::utils::weak_component_link::WeakComponentLink;

#[derive(PartialEq, Properties)]
pub struct CrosswordComponentProps
{
    pub crossword: Crossword<u8, String>,
    pub link: WeakComponentLink<CrosswordComponent>
}

pub enum CrosswordComponentMessage
{
    Set(Crossword<u8, String>),
    AddWord(PlacedWord<u8, String>),
    RemoveWord(String)
}

pub struct CrosswordComponent
{
    crossword: Crossword<u8, String>
}

impl Component for CrosswordComponent
{
    type Properties = CrosswordComponentProps;
    type Message = CrosswordComponentMessage;

    fn create(ctx: &Context<Self>) -> Self
    {
        log!("Crossword Create");
        ctx.props().link.borrow_mut().replace(ctx.link().clone());
        CrosswordComponent
        {
            crossword: ctx.props().crossword.clone()
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool 
    {
        log!("Crossword Update");
        match msg
        {
            CrosswordComponentMessage::Set(cw) => self.crossword = cw,
            CrosswordComponentMessage::AddWord(w) => { self.crossword.add_word(w); },
            CrosswordComponentMessage::RemoveWord(s) => { self.crossword.remove_word(&s); }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool 
    {
        log!("Crossword Changed");
        if ctx.props().crossword != _old_props.crossword
        {
            ctx.link().send_message(CrosswordComponentMessage::Set(ctx.props().crossword.clone()));
            return false;
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html
    {
        log!("Crossword View");
        let cw = &ctx.props().crossword;
        let size_x = cw.get_size().0;
        let grid_item_html_iter = cw.generate_char_table()
            .into_iter()                                                // iter(vec(u8))
            .map(|r| 
                r.into_iter()
                .enumerate())                                           // iter(iter(usize, u8))
            .enumerate()                                                // iter((usize, iter(usize, u8)))
            .map(|(i, iter)| 
                repeat(i).zip(iter))                                    // iter(iter((usize, (usize, u8))))
            .flatten()                                                  // iter((usize, (usize, u8)))
            .map(|(y, (x, ch))| ((x, y), ch))         // iter(((usize, usize), u8))
            .filter(|((_, _), ch)| *ch != u8::default())
            .map(|((x, y), ch)| 
            html!
            {
                <div class={classes!("crosswor-grid-item",
                    css!( grid-area: ${y + 1} / ${x + 1} / span 1 / span 1; ), //grid-item
                    css!(
                        aspect-ratio: 1/1;
                        text-align: center;
                        align-content: center;
                        background-color: rgb(84, 84, 84);
                        color: white;
                        border-radius: 5px;
                    ))}
                >
                    {ch as char}
                </div>
            });                          

        html!
        {
            <div class={classes!("crossword-grid",
                css!( grid-template-columns: repeat(${size_x}, 1fr); ), //grid
                css!(
                    display: grid;
                    gap: 5px;
                    background-color: rgb(156, 156, 156);
                    padding: 10px;
                    border-radius: 10px;
                    font-size: 25px;
                    max-width: 100%;
                    box-sizing: border-box;
                ))}
            >
                {for grid_item_html_iter}
            </div>
        }
    }
}