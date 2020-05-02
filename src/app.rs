use std::f64;
use std::fs::File;
use std::io::prelude::*;

use cursive::direction::{Absolute, Direction};
use cursive::event::{Event, EventResult, Key};
use cursive::view::View;
use cursive::{Printer, Vec2};

use chrono::NaiveDate;

use crate::habit::{Bit, Count, Habit, HabitWrapper};
use crate::CONFIGURATION;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize)]
pub enum ViewMode {
    Day,
    Month,
    Year,
}

impl std::default::Default for ViewMode {
    fn default() -> Self {
        ViewMode::Month
    }
}

#[derive(Serialize, Deserialize)]
pub struct App {
    habits: Vec<Box<dyn HabitWrapper>>,
    focus: usize,

    #[serde(skip)]
    view_mode: ViewMode,
}

impl App {
    pub fn new() -> Self {
        return App {
            habits: vec![],
            view_mode: ViewMode::Day,
            focus: 0,
        };
    }

    pub fn add_habit(&mut self, h: Box<dyn HabitWrapper>) {
        self.habits.push(h);
    }

    pub fn set_mode(&mut self, set_mode: ViewMode) {
        if set_mode != self.view_mode {
            self.view_mode = set_mode;
        }
    }

    fn set_focus(&mut self, d: Absolute) {
        let grid_width = CONFIGURATION.grid_width;
        match d {
            Absolute::Right => {
                if self.focus != self.habits.len() - 1 {
                    self.focus += 1;
                }
            }
            Absolute::Left => {
                if self.focus != 0 {
                    self.focus -= 1;
                }
            }
            Absolute::Down => {
                if self.focus + grid_width < self.habits.len() - 1 {
                    self.focus += grid_width;
                } else {
                    self.focus = self.habits.len() - 1;
                }
            }
            Absolute::Up => {
                if self.focus as isize - grid_width as isize >= 0 {
                    self.focus -= grid_width;
                } else {
                    self.focus = 0;
                }
            }
            Absolute::None => {}
        }
    }

    fn status(&self) -> String {
        let today = chrono::Local::now().naive_utc().date();
        let remaining = self.habits.iter().map(|h| h.remaining(today)).sum::<u32>();
        let total = self.habits.iter().map(|h| h.total()).sum::<u32>();
        let completed = total - remaining;

        return format!("{} completed, {} remaining", completed, remaining);
    }

    fn max_size(&self) -> Vec2 {
        let grid_width = CONFIGURATION.grid_width;
        let width = {
            if self.habits.len() > 0 {
                grid_width * CONFIGURATION.view_width
            } else {
                0
            }
        };
        let height = {
            if self.habits.len() > 0 {
                (CONFIGURATION.view_height as f64
                    * (self.habits.len() as f64 / grid_width as f64).ceil())
                    as usize
            } else {
                0
            }
        };
        Vec2::new(width, height)
    }

    fn load_state() -> Self {
        let mut file = File::open("foo.txt").unwrap();
        let mut j = String::new();
        file.read_to_string(&mut j);
        return serde_json::from_str(&j).unwrap();
    }

    // this function does IO
    // TODO: convert this into non-blocking async function
    fn save_state(&self) {
        let j = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create("foo.txt").unwrap();
        file.write_all(j.as_bytes()).unwrap();
    }
}

impl View for App {
    fn draw(&self, printer: &Printer) {
        let grid_width = CONFIGURATION.grid_width;
        let mut offset = Vec2::zero();
        for (idx, i) in self.habits.iter().enumerate() {
            if idx >= grid_width && idx % grid_width == 0 {
                offset = offset.map_y(|y| y + CONFIGURATION.view_height).map_x(|_| 0);
            }
            i.draw(&printer.offset(offset).focused(self.focus == idx));
            offset = offset.map_x(|x| x + CONFIGURATION.view_width);
        }
        offset = offset.map_x(|_| 0).map_y(|_| self.max_size().y - 2);
        printer.print(offset, &self.status());
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        let grid_width = CONFIGURATION.grid_width;
        let width = {
            if self.habits.len() > 0 {
                grid_width * CONFIGURATION.view_width
            } else {
                0
            }
        };
        let height = {
            if self.habits.len() > 0 {
                (CONFIGURATION.view_height as f64
                    * (self.habits.len() as f64 / grid_width as f64).ceil())
                    as usize
            } else {
                0
            }
        };
        Vec2::new(width, height)
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        false
    }

    fn on_event(&mut self, e: Event) -> EventResult {
        match e {
            Event::Key(Key::Right) | Event::Key(Key::Tab) | Event::Char('l') => {
                self.set_focus(Absolute::Right);
                return EventResult::Consumed(None);
            }
            Event::Key(Key::Left) | Event::Shift(Key::Tab) | Event::Char('h') => {
                self.set_focus(Absolute::Left);
                return EventResult::Consumed(None);
            }
            Event::Key(Key::Up) | Event::Char('k') => {
                self.set_focus(Absolute::Up);
                return EventResult::Consumed(None);
            }
            Event::Key(Key::Down) | Event::Char('j') => {
                self.set_focus(Absolute::Down);
                return EventResult::Consumed(None);
            }
            Event::Char('a') => {
                let mut gymming = Count::new("gym", 5);
                gymming.insert_entry(NaiveDate::from_ymd(2020, 4, 11), 7);
                gymming.insert_entry(NaiveDate::from_ymd(2020, 4, 12), 8);
                gymming.insert_entry(NaiveDate::from_ymd(2020, 4, 13), 9);
                gymming.insert_entry(NaiveDate::from_ymd(2020, 4, 14), 10);
                gymming.insert_entry(NaiveDate::from_ymd(2020, 4, 15), 11);
                self.add_habit(Box::new(gymming));
                return EventResult::Consumed(None);
            }
            Event::Char('d') => {
                self.habits.remove(self.focus);
                return EventResult::Consumed(None);
            }
            Event::Char('w') => {
                let j = serde_json::to_string_pretty(&self).unwrap();
                let mut file = File::create("foo.txt").unwrap();
                file.write_all(j.as_bytes()).unwrap();
                return EventResult::Consumed(None);
            }
            Event::Char('q') => {
                self.save_state();
                return EventResult::with_cb(|s| s.quit());
            }
            _ => self.habits[self.focus].on_event(e),
        }
    }
}