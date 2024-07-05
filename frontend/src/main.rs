mod components;

use components::crossword_component::CrosswordComponent;
use crossword_generator::{crossword::Crossword, placed_word::PlacedWord, word::{Direction, Position}};
use yew::prelude::*;

#[function_component]
fn App() -> Html {

    let mut cw = Crossword::default();   
    cw.add_word(PlacedWord::<u8, String>::new( "hello".to_owned(), Position { x: 0, y: 0 }, Direction::Right)).unwrap();
    cw.add_word(PlacedWord::<u8, String>::new( "local".to_owned(), Position { x: 2, y: 0 }, Direction::Down)).unwrap();
    cw.add_word(PlacedWord::<u8, String>::new( "cat".to_owned(), Position { x: 2, y: 2 }, Direction::Right)).unwrap();


    html! {
    <>
        <p>{"Hello from this side"}</p>
        <div style="width: 400px;">
            <CrosswordComponent crossword={cw}/> 
        </div>
    </>
    }
}

fn main() {

    yew::Renderer::<App>::new().render();
}