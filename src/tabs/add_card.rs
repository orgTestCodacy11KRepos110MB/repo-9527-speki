use crate::app::AppData;
use crate::app::Tab;
use crate::app::TabData;
use crate::app::Widget;
use crate::utils::card::CardView;
use crate::utils::misc::{split_leftright_by_percent, split_updown_by_percent};
use crate::widgets::button::Button;
use crate::{MyKey, MyType};
use rusqlite::Connection;
use tui::layout::Rect;
use tui::Frame;

//#[derive(Clone)]
pub struct NewCard {
    pub prompt: Button,
    cardview: CardView,
    tabdata: TabData,
}

use std::sync::{Arc, Mutex};

impl NewCard {
    pub fn new(appdata: &AppData) -> NewCard {
        let cardview = CardView::new(appdata);

        NewCard {
            prompt: Button::new("Add new card".to_string()),
            cardview,
            tabdata: TabData::default(),
        }
    }

    fn make_prompt(_conn: &Arc<Mutex<Connection>>) -> String {
        "Add new card".to_string()
    }

    pub fn submit_card(&mut self, appdata: &AppData, iscompleted: bool) {
        self.cardview.submit_card(appdata, iscompleted);
        *self = Self::new(appdata);
    }
}

impl Tab for NewCard {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn get_title(&self) -> String {
        "Add card".to_string()
    }
    fn get_manual(&self) -> String {
        r#"

Topic of card is as selected in the topic widget.

Upper textbox is question, lower is answer.

add card as finished: Alt+f
Add card as unfinished: Alt+u    

        "#
        .to_string()
    }

    fn set_selection(&mut self, area: Rect) {
        let chunks = split_leftright_by_percent([75, 15], area);
        let left = chunks[0];
        let right = chunks[1];
        let chunks = split_updown_by_percent([10, 37, 37], left);

        self.tabdata.view.areas.push(chunks[0]);
        self.tabdata.view.areas.push(right);
        self.tabdata.view.areas.push(chunks[1]);
        self.tabdata.view.areas.push(chunks[2]);

        self.prompt.set_area(chunks[0]);
        self.cardview.question.set_area(chunks[1]);
        self.cardview.answer.set_area(chunks[2]);
        self.cardview.topics.set_area(right);
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey, cursor: &(u16, u16)) {
        match key {
            MyKey::Alt('f') => self.submit_card(appdata, true),
            MyKey::Alt('u') => self.submit_card(appdata, false),
            key if self.cardview.is_selected(cursor) => {
                self.cardview
                    .keyhandler(appdata, &mut self.tabdata, cursor, key)
            }
            _ => {}
        }
    }
    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &(u16, u16)) {
        self.prompt.render(f, appdata, cursor);
        self.cardview.render(f, appdata, cursor);
    }
}