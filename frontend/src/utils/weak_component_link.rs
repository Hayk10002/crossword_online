use std::{cell::RefCell, ops::Deref, rc::Rc};

use yew::{html::{ImplicitClone, Scope}, Component};

pub struct WeakComponentLink<COMP: Component>(Rc<RefCell<Option<Scope<COMP>>>>);

impl<COMP: Component> Clone for WeakComponentLink<COMP> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
impl<COMP: Component> ImplicitClone for WeakComponentLink<COMP> {}

impl<COMP: Component> Default for WeakComponentLink<COMP> {
    fn default() -> Self {
        Self(Rc::default())
    }
}

impl<COMP: Component> Deref for WeakComponentLink<COMP> {
    type Target = Rc<RefCell<Option<Scope<COMP>>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<COMP: Component> PartialEq for WeakComponentLink<COMP> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}