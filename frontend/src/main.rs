mod components;
mod utils;

use std::default;

use components::{playground_component::PlaygroundComponent, word_component::WordComponent};
use crossword_generator::{crossword::{Crossword, WordCompatibilitySettings}, placed_word::PlacedWord, word::{Direction, Position, Word}};
use stylist::{css, global_style, yew::Global, Style};
use utils::{settings::StyleSettings, weak_component_link::WeakComponentLink};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
fn App() -> Html 
{

    let mut settings = WordCompatibilitySettings::default();
    settings.corner_by_corner = false;
    let cw = use_state(|| vec![
        PlacedWord::<char, Vec<char>>::new( "hello".chars().collect(), Position { x: 0, y: 0 }, Direction::Right),
        PlacedWord::<char, Vec<char>>::new( "local".chars().collect(), Position { x: 1, y: 0 }, Direction::Down),
        PlacedWord::<char, Vec<char>>::new( "at".chars().collect(), Position { x: 3, y: 3 }, Direction::Right),
        PlacedWord::<char, Vec<char>>::new( "table".chars().collect(), Position { x: 5, y: 3 }, Direction::Right),
        PlacedWord::<char, Vec<char>>::new( "sezam".chars().collect(), Position { x: 10, y: -2 }, Direction::Down),
        PlacedWord::<char, Vec<char>>::new( "sezon".chars().collect(), Position { x: 12, y: -2 }, Direction::Down),
        PlacedWord::<char, Vec<char>>::new( "abcde".chars().collect(), Position { x: 9, y: 0 }, Direction::Right),
    ]);

    let cw_link = use_state(|| WeakComponentLink::<PlaygroundComponent<char, Vec<char>>>::default());


    let w = Word::new("Helloworld".chars().collect(), None);

    let w_link = use_state(|| WeakComponentLink::<WordComponent>::default());
        
    let w2 = Word::new("Helloworld".chars().collect(), None);

    let w_link2 = use_state(|| WeakComponentLink::<WordComponent>::default());

    let placed_words_html = (0..(*cw).len()).map(|i|
    html! {
        <form class={classes!(css!( display: flex; flex-wrap: wrap; ))}>
            <input type="text" value={(*cw)[i].value.iter().cloned().collect::<String>()}
            onchange={
                let cw = cw.clone();
                Callback::from(move |event: Event| 
                {
                    let mut cw_clone = (*cw).clone();
                    cw_clone[i].value = event.target_dyn_into::<HtmlInputElement>().unwrap().value().chars().collect();
                    cw.set(cw_clone);
                })
            }
            />
            <input type="number" value={format!("{}", (*cw)[i].position.x)}
            onchange={
                let cw = cw.clone();
                Callback::from(move |event: Event| 
                {
                    let mut cw_clone = (*cw).clone();
                    cw_clone[i].position.x = event.target_dyn_into::<HtmlInputElement>().unwrap().value().parse::<i16>().unwrap();
                    cw.set(cw_clone);
                })
            }/>
            <input type="number" value={format!("{}", (*cw)[i].position.y)}
            onchange={
                let cw = cw.clone();
                Callback::from(move |event: Event| 
                {
                    let mut cw_clone = (*cw).clone();
                    cw_clone[i].position.y = event.target_dyn_into::<HtmlInputElement>().unwrap().value().parse::<i16>().unwrap();
                    cw.set(cw_clone);
                })
            }/>
            <input type="radio" name="direction" checked={(*cw)[i].direction == Direction::Right}
            onchange={
                let cw = cw.clone();
                Callback::from(move |_| 
                {
                    let mut cw_clone = (*cw).clone();
                    cw_clone[i].direction = Direction::Right;
                    cw.set(cw_clone);
                })
            }/>
            <input type="radio" name="direction" checked={(*cw)[i].direction == Direction::Down}
            onchange={
                let cw = cw.clone();
                Callback::from(move |_| 
                {
                    let mut cw_clone = (*cw).clone();
                    cw_clone[i].direction = Direction::Down;
                    cw.set(cw_clone);
                })
            }/>
        </form>
    });

    let style_settings = use_state(|| StyleSettings::new()); 

    html! {
    <>
        <Global css={css!(
            *
            {
                //outline: 1px red solid;
            }
            body
            {
                width: 100vw;
                height: 100vh;
                margin: 0px;
                padding: 8px;
                box-sizing: border-box;
                overflow: hidden;
                background-color: rgb(84, 84, 84);
            }
        )}/>
        <ContextProvider<StyleSettings> context={(*style_settings).clone()}>
            <div class={classes!("web-layout", 
                css!
                (
                    display: grid;
                    grid-template-areas: ${"\'sidebar playground\'"};
                    grid-template-columns: 400px auto;
                    box-sizing: border-box;
                    height: 600px;   
                )
            )}>
                <div class={classes!("sidebar", css!( grid-area: sidebar; ))}>
                    <p>{"Hello from this side"}</p>
                    <WordComponent word={w} link={(*w_link).clone()}/>
                    <WordComponent word={w2} link={(*w_link2).clone()}/>

                    { for placed_words_html }
                </div>
                <div class={classes!("playground-area", css!( grid-area: playground; ))}>
                    <PlaygroundComponent<char, Vec<char>> words={(*cw).clone()} word_compatibility_settings={settings} link={(*cw_link).clone()}/>
                </div>
            </div>
        </ContextProvider<StyleSettings>>
    </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}