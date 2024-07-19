#![allow(non_upper_case_globals)] 

use crossword_generator::word::{Direction, Word};
use gloo_console::log;
use yew::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use stylist::css;

use super::super::utils::weak_component_link::WeakComponentLink;

#[derive(PartialEq, Properties)]
pub struct WordComponentProps
{
    pub word: Word<u8, String>,
    pub link: WeakComponentLink<WordComponent>
}

pub enum WordComponentMessage
{
    Set(Word<u8, String>),
    SetString(String),
    SetDirection(Option<Direction>)
}

pub struct WordComponent
{
    word: Word<u8, String>
}

impl Component for WordComponent
{
    type Properties = WordComponentProps;
    type Message = WordComponentMessage;

    fn create(ctx: &Context<Self>) -> Self
    {
        log!("Word Create");
        ctx.props().link.borrow_mut().replace(ctx.link().clone());
        WordComponent
        {
            word: ctx.props().word.clone()
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool 
    {
        log!("Word Update");
        match msg
        {
            WordComponentMessage::Set(word) => self.word = word,
            WordComponentMessage::SetString(str) => self.word.value = str,
            WordComponentMessage::SetDirection(dir) => self.word.dir = dir
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool 
    {
        log!("Word Changed");
        if ctx.props().word != _old_props.word
        {
            ctx.link().send_message(WordComponentMessage::Set(ctx.props().word.clone()));
            return false;
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html
    {
        log!("Word View");
        

        let onchange = 
        {
            let ctx_link = ctx.link().clone();
            Callback::from(move |e: Event| 
            {
                ctx_link.send_message(WordComponentMessage::SetString(e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap().value()));
            })
        };

        

        html! 
        { 
            <form> // to use the same name for all radios for all words
                <div class={classes!("word-container-wrapper",
                    css!(
                        padding: 10px;
                        border-radius: 10px;
                        background-color: rgb(156, 156, 156);

                        :hover:not(:has(.hoverable:hover))
                        {
                            background-color: rgb(125, 125, 125);
                        }
                    ))}
                >
                    
                    <div class={classes!("word-container-flex",
                        css!(
                            display: flex;
                            flex-wrap: wrap;
                            padding: 10px;
                            gap: 10px;
                        ))}
                    >
                        <div class={classes!("word-input-wrapper",
                            css!(
                                flex-shrink: 1;
                                flex-grow: 1;
                                flex-basis: 200px;
                            ))}
                        >
                            <input type="text" placeholder="Enter a word" value={self.word.value.clone()} {onchange} class={classes!("word-input", "hoverable", 
                                css!(
                                    width: 100%;
                                    box-sizing: border-box;
                                    border: 0px;
                                    border-radius: 9999px;
                                    font-size: 20px;
                                    background-color: rgb(84, 84, 84);
                                    color: white;
                                    padding: 12px 20px 12px 20px;
            
                                    ::placeholder 
                                    {
                                        color: rgb(156, 156, 156);
                                    }
            
                                ))}
                            />
                        </div>
                        <div class={classes!("word-direction-wrapper",
                            css!(
                                flex-basis: 100px;
                                flex-shrink: 1;
                                display: flex;
                                align-items: center;
                            ))}
                        >
                        { 
                            for [(Some(Direction::Right), "right"), (Some(Direction::Down), "down"), (None, "both")].into_iter().enumerate().map(|(i, (direction, arrow_type))| html! 
                            {
                                <label class={classes!(format!("word-direction-{}-label", arrow_type), "hoverable", 
                                    css!( 
                                        background-image: url(${format!("/data/images/word_direction_{}_arrow.png", arrow_type)}); 
                                        background-color: rgb(156, 156, 156);
                                        border-width: 2px ${" "} ${ if i >= 1 { 2 } else { 0 }}px 2px ${" "} ${ if i <= 1 { 2 } else { 0 }}px;
                                        border-radius: ${ if i == 0 { 5 } else { 0 }}px ${" "} ${ if i == 2 { 5 } else { 0 }}px ${" "} ${ if i == 2 { 5 } else { 0 }}px ${" "} ${ if i == 0 { 5 } else { 0 }}px;
                                    ),
                                    css!(
                                        display: block;
                                        background-size: contain;
                                        cursor: pointer;
                                        width: 100%;
                                        aspect-ratio: 1;
                                        border-style: solid;
                                        border-color: rgb(84, 84, 84);

                                        :hover
                                        {
                                            background-color: rgb(125, 125, 125);
                                        }

                                        :has(> input:checked)
                                        {
                                            background-color: rgb(84, 84, 84);
                                        }
                                    ))}
                                > // to be able to click on pictures
                                    <input type="radio" name="direction" checked={direction == self.word.dir} class={classes!(format!("word-direction-{}-input", arrow_type),
                                        css!(
                                            position: absolute;
                                            cursor: pointer;
                                            opacity: 0;
                                        ))}
                                        onchange=
                                        {
                                            let ctx_link = ctx.link().clone();
                                            Callback::from(move |_| 
                                            {
                                                ctx_link.send_message(WordComponentMessage::SetDirection(direction.clone()));
                                            })
                                        }
                                    />
                                </label>
                            })
                        }
                        </div>
                    </div>
                </div>
            </form>
        }
    }
}