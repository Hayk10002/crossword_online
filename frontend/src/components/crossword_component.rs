use std::iter::repeat;

use crossword_generator::crossword::Crossword;
use yew::prelude::*;
use stylist::yew::styled_component;

#[derive(PartialEq, Properties)]
pub struct CrosswordComponentProps
{
    pub crossword: Crossword<u8, String>
}

#[styled_component]
pub fn CrosswordComponent(props: &CrosswordComponentProps) -> Html
{
    
    let cw = &props.crossword;
    let size_x = cw.get_size().0;
    let char_hashmap = cw.generate_char_table()
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
            <div class={
                css!(
                    grid-area: ${y + 1} / ${x + 1} / span 1 / span 1;
                    aspect-ratio: 1/1;
                    text-align: center;
                    align-content: center;
                    background-color: rgb(84, 84, 84);
                    color: white;
                    border-radius: 5px;
                )}>
                {ch as char}
            </div>
        });                          

    
    html!
    {
        <div class={
            css!(
                display: grid;
                grid-template-columns: repeat(${size_x}, 1fr);
                gap: 5px;
                background-color: rgb(156, 156, 156);
                padding: 10px;
                border-radius: 10px;
                font-size: 25px;
                max-width: 100%;
                box-sizing: border-box;
            )}>
            {for char_hashmap}
        </div>
    }
}