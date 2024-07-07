mod components;
mod utils;

use std::default;

use components::crossword_component::CrosswordComponent;
use crossword_generator::{crossword::Crossword, placed_word::PlacedWord, word::{Direction, Position}};
use utils::weak_component_link::WeakComponentLink;
use yew::prelude::*;

#[function_component]
fn App() -> Html {

    let mut cw1 = Crossword::default();   
    cw1.add_word(PlacedWord::<u8, String>::new( "hello".to_owned(), Position { x: 0, y: 0 }, Direction::Right)).unwrap();
    cw1.add_word(PlacedWord::<u8, String>::new( "local".to_owned(), Position { x: 2, y: 0 }, Direction::Down)).unwrap();
    cw1.add_word(PlacedWord::<u8, String>::new( "cat".to_owned(), Position { x: 2, y: 2 }, Direction::Right)).unwrap();

    let mut cw2 = Crossword::default();   
    cw2.add_word(PlacedWord::<u8, String>::new( "hello".to_owned(), Position { x: 0, y: 0 }, Direction::Down)).unwrap();
    cw2.add_word(PlacedWord::<u8, String>::new( "local".to_owned(), Position { x: 0, y: 2 }, Direction::Right)).unwrap();
    cw2.add_word(PlacedWord::<u8, String>::new( "cat".to_owned(), Position { x: 2, y: 2 }, Direction::Down)).unwrap();

    let cw = use_state(|| cw1.clone());

    let link = use_state(|| WeakComponentLink::<CrosswordComponent>::default());
    let onclick = 
    {
        let cw = cw.clone(); 
        let cw1 = cw1.clone();
        let cw2 = cw2.clone();
        Callback::from(move |_| if *cw == cw1 { cw.set(cw2.clone()); } else if *cw == cw2 { cw.set(cw1.clone()); })
    };

    html! {
    <>
        <p>{"Hello from this side"}</p>
        <button {onclick}>{"Change crosswords"}</button>
        <div style="width: 400px;">
            <CrosswordComponent crossword={(*cw).clone()} link={(*link).clone()}/>
        </div>
    </>
    }
}

fn main() {

    yew::Renderer::<App>::new().render();
}