use super::Rect;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::stdout;
pub const FG: Color = Color::White;
pub const BG: Color = Color::Rgb { r: 0, g: 0, b: 0 };
#[derive(Clone, PartialEq, Eq)]
pub enum ContKind{
    Empty,
    Grapheme(String),
    Continue,
}
#[derive(Clone, PartialEq, Eq)]
struct Cell {
    c: ContKind,
    fg: Color,
    bg: Color,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ContKind::Empty,
            fg: FG,
            bg: BG,
        }
    }
}
impl Cell{
    pub fn new(c:ContKind, fg:Color, bg:Color)->Cell{
        Cell { c, fg, bg }
    }
    fn grapheme(s:&str, fg:Color, bg:Color)->Self{
        Self{
            c:ContKind::Grapheme(s.into()),
            fg,
            bg,
        }
    }
    fn continuation(fg:Color, bg:Color)->Self{
        Self{
            c:ContKind::Continue,
            fg,
            bg,
        }
    }
}
pub struct ScreenBuffer {
    pub cells: Vec<Cell>,
    pub w: u16,
    pub h: u16,
}

use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;
impl ScreenBuffer {
    pub fn new(w:u16, h:u16)->Self{
        Self { cells: vec![Cell::default(); (w*h) as usize], w, h}
    }
    fn clear_buffer(&mut self) {
        self.cells.fill(Cell::default())
    }
    pub fn set<'a>(&mut self, r: Rect, x: u16, y: u16, s: Box<str>, fg: Color, bg: Color) {
        let mut c = x;
        for grapheme in s.graphemes(true){
            let w = grapheme.width() as u16;
            if w == 0 {continue;}
            if c >= r.right() || c >= self.w{
                break
            }
            if c + w > r.right() || c + w > self.w{
                break
            }
            self.set_cell(r, c, y, Cell { c: ContKind::Grapheme(grapheme.to_string()), fg, bg });
            for o in 1..w{
                self.set_cell(r, c + o, y, Cell { c: ContKind::Continue, fg, bg });
            }
            c += w;
        }
    }
    fn clear_grapheme_at(&mut self, x: u16, y: u16) {
        if x >= self.w || y >= self.h { return; }
        let idx = (y as usize * self.w as usize) + x as usize;
        let start_x = match &self.cells[idx].c {
            ContKind::Continue => x.saturating_sub(1),
            ContKind::Grapheme(_) => x,
            ContKind::Empty => return,
        };
        let start_idx = (y as usize * self.w as usize) + start_x as usize;
        let width = match &self.cells[start_idx].c {
            ContKind::Grapheme(s) => s.width() as u16,
            _ => return,
        };
        for offset in 0..width {
            let clear_idx = (y as usize * self.w as usize) + (start_x + offset) as usize;
            self.cells[clear_idx] = Cell::default();
        }
    }
    fn set_cell(&mut self, r: Rect, x: u16, y: u16, cell: Cell) {
        if r.w == 0
            || r.h == 0
            || x < r.x
            || y < r.y
            || x > r.right()
            || y > r.bottom()
            || x >= self.w
            || y >= self.h
        {
            return;
        }
        let idx = y * self.w + x;
        self.cells[idx as usize] = cell;
    }
    pub fn draw(&mut self, old: &mut ScreenBuffer) -> std::io::Result<()> {
        let mut out = stdout().lock();
        for y in 0..self.h {
            let mut x = 0;
            while x < self.w {
                let idx = (y as usize * self.w as usize) + x as usize;
                let new = &self.cells[idx];
                let prev = &old.cells[idx];
                if new == prev {
                    x += 1;
                    continue;
                }
                if matches!(new.c, ContKind::Continue) {
                    x += 1;
                    continue;
                }
                queue!(
                    out,
                    MoveTo(x, y),
                    SetForegroundColor(new.fg),
                    SetBackgroundColor(new.bg)
                )?;
                match &new.c {
                    ContKind::Grapheme(s) => {
                        queue!(out, Print(s))?;
                        x += s.width() as u16;
                    }
                    ContKind::Empty => {
                        queue!(out, Print(" "))?;
                        x += 1;
                    }
                    ContKind::Continue => unreachable!(),
                }
            }
        }
        queue!(out, ResetColor)?;
        std::mem::swap(&mut self.cells, &mut old.cells);
        self.clear_buffer();
        Ok(())
    }
}

