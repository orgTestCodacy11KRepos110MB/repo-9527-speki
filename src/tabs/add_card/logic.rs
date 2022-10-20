use crate::app::AppData;
use crate::app::Tab;
use crate::utils::card::CardType;
use crate::utils::misc::get_gpt3_response;
use crate::utils::misc::{split_leftright_by_percent, split_updown_by_percent};
use crate::utils::{aliases::*, sql::fetch::load_inc_title};
use crate::utils::{card::Card, sql::fetch::fetch_card};
use crate::widgets::find_card::FindCardWidget;
use crate::widgets::message_box::draw_message;
use crate::widgets::textinput::Field;
use crate::{MyKey, MyType};
use rusqlite::Connection;
use tui::layout::Rect;
use tui::style::Style;
use tui::Frame;

#[derive(Clone)]
pub enum DepState {
    None,
    NewDependent(CardID),
    NewDependency(CardID),
    NewChild(IncID),
}

//#[derive(Clone)]
pub enum TextSelect {
    Question, // Bool indicates if youre in text-editing mode
    Answer,
    Topic,
    ChooseCard(FindCardWidget),
}

use crate::widgets::topics::TopicList;

//#[derive(Clone)]
pub struct NewCard {
    pub prompt: String,
    pub question: Field,
    pub answer: Field,
    pub state: DepState,
    pub topics: TopicList,
    pub selection: TextSelect,
}

use std::sync::{Arc, Mutex};

impl NewCard {
    pub fn new(conn: &Arc<Mutex<Connection>>, state: DepState) -> NewCard {
        let mut topics = TopicList::new(conn);
        topics.next();

        NewCard {
            prompt: NewCard::make_prompt(&state, conn),
            question: Field::new(),
            answer: Field::new(),
            state,
            topics,
            selection: TextSelect::Question,
        }
    }

    pub fn navigate(&mut self, dir: crate::Direction) {
        use crate::Direction::*;
        use TextSelect::*;
        match (&self.selection, dir) {
            (Question, Right) => self.selection = Topic,
            (Question, Down) => self.selection = Answer,
            (Answer, Up) => self.selection = Question,
            (Answer, Right) => self.selection = Topic,
            (Topic, Left) => self.selection = Question,
            (_, _) => {}
        }
    }

    fn make_prompt(state: &DepState, conn: &Arc<Mutex<Connection>>) -> String {
        let mut prompt = String::new();
        match state {
            DepState::None => {
                prompt.push_str("Add new card");
                prompt
            }
            DepState::NewDependency(idx) => {
                prompt.push_str("Add new dependency for ");
                let card = fetch_card(conn, *idx);
                prompt.push_str(&card.question);
                prompt
            }
            DepState::NewDependent(idx) => {
                prompt.push_str("Add new dependent of: ");
                let card = fetch_card(conn, *idx);
                prompt.push_str(&card.question);
                prompt
            }
            DepState::NewChild(id) => {
                prompt.push_str("Add new child of source: ");
                let title = load_inc_title(conn, *id, 15).unwrap();
                prompt.push_str(&title);
                prompt
            }
        }
    }

    pub fn submit_card(&mut self, conn: &Arc<Mutex<Connection>>, iscompleted: bool) {
        let question = self.question.return_text();
        let answer = self.answer.return_text();
        let topic = self.topics.get_selected_id().unwrap();
        let source = if let DepState::NewChild(incid) = self.state {
            incid
        } else {
            0
        };
        let status = if iscompleted {
            CardType::Finished
        } else {
            CardType::Unfinished
        };

        //(conn, question, answer, topic, source, iscompleted);
        let mut card = Card::new()
            .question(question)
            .answer(answer)
            .topic(topic)
            .source(source)
            .cardtype(status);

        //   revlog_new(conn, highest_id(conn).unwrap(), Review::from(&RecallGrade::Decent)).unwrap();

        match self.state {
            DepState::None => {}
            DepState::NewDependency(id) => {
                card.dependency(id);
            }
            DepState::NewDependent(id) => {
                card.dependent(id);
            }
            DepState::NewChild(_id) => {}
        }

        card.save_card(conn);
        //self.reset(DepState::None, conn);
        *self = Self::new(conn, DepState::None);
    }

    pub fn uprow(&mut self) {}
    pub fn downrow(&mut self) {}
    pub fn home(&mut self) {}
    pub fn end(&mut self) {}
}

impl Tab for NewCard {
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
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        use MyKey::*;
        use TextSelect::*;
        match (&self.selection, key) {
            (_, Nav(dir)) => self.navigate(dir),
            (_, Alt('f')) => self.submit_card(&appdata.conn, true),
            (_, Alt('u')) => self.submit_card(&appdata.conn, false),
            (_, Alt('g')) => {
                if let Some(key) = &appdata.config.gptkey {
                    let answer = get_gpt3_response(key, &self.question.return_text());
                    self.answer.replace_text(answer);
                }
            }
            (Question, key) => self.question.keyhandler(key),
            (Answer, key) => self.answer.keyhandler(key),
            (Topic, key) => self.topics.keyhandler(key, &appdata.conn),
            (_, _) => {}
        }
    }
    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, area: Rect) {
        let chunks = split_leftright_by_percent([75, 15], area);
        let left = chunks[0];
        let right = chunks[1];

        self.topics.render(
            f,
            right,
            matches!(&self.selection, TextSelect::Topic),
            "Topics",
            Style::default(),
        );

        let chunks = split_updown_by_percent([10, 37, 37], left);

        draw_message(f, chunks[0], self.prompt.as_str());
        self.question.render(
            f,
            chunks[1],
            matches!(&self.selection, TextSelect::Question),
        );
        self.answer
            .render(f, chunks[2], matches!(&self.selection, TextSelect::Answer));
    }
}
