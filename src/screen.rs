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
    s: Style,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Style{
    fg:Color,
    bg:Color,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ContKind::Empty,
            s: Style { fg: FG, bg: BG },
        }
    }
}
impl Cell{
    fn grapheme(s:&str, fg:Color, bg:Color)->Self{
        Self{
            c:ContKind::Grapheme(s.into()),
            s:Style { fg, bg },
        }
    }
    fn continuation(fg:Color, bg:Color)->Self{
        Self{
            c:ContKind::Continue,
            s:Style { fg, bg },
        }
    }
}
pub struct ScreenBuffer {
    cells: Vec<Cell>,
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
    pub fn set(
        &mut self,
        r: Rect,
        x: u16,
        y: u16,
        s: &str,
        fg: Color,
        bg: Color,
    ) {
        if r.w == 0 || r.h == 0 { return; }
        if y < r.y || y >= r.bottom() || y >= self.h { return; }
        let right = r.right().min(self.w);
        let mut cx = x;
        for grapheme in s.graphemes(true) {
            let width = grapheme.width();
            if width == 0 { continue; }
            let width = width as usize;
            if cx < r.x { continue; }
            let end = cx as usize + width;
            if cx >= right || end > right as usize {
                break;
            }
            // Clear any existing grapheme overlapping the new one.
            for col in cx..end as u16 {
                self.clear_grapheme(col, y);
            }
            let idx = y as usize * self.w as usize + cx as usize;
            self.cells[idx] = Cell::grapheme(grapheme, fg, bg);
            for offset in 1..width {
                let idx = y as usize * self.w as usize + cx as usize + offset;
                self.cells[idx] = Cell::continuation(fg, bg);
            }
            cx = end as u16;
        }
    }
    fn clear_grapheme(&mut self, x: u16, y: u16) {
        if x >= self.w || y >= self.h { return; }
        let row_start = y as usize * self.w as usize;
        let mut start_x = x;
        while start_x > 0 {
            let idx = row_start + start_x as usize;
            match self.cells[idx].c {
                ContKind::Continue => start_x -= 1,
                ContKind::Grapheme(_) | ContKind::Empty => break,
            }
        }
        let idx = row_start + start_x as usize;
        let width = match &self.cells[idx].c {
            ContKind::Grapheme(s) => s.width(),
            _ => return,
        };
        for offset in 0..width {
            let idx = row_start + start_x as usize + offset;
            self.cells[idx] = Cell::default();
        }
    }
    pub fn draw(&mut self, old: &mut ScreenBuffer) -> std::io::Result<()> {
        use std::io::Write;
        let mut out = stdout().lock();
        let (mut curr_fg, mut curr_bg) = (None, None);
        for y in 0..self.h {
            let mut x = 0;
            while x < self.w {
                let idx = y as usize * self.w as usize + x as usize;
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
                queue!(out, MoveTo(x, y))?;
                if curr_fg != Some(new.s.fg) {
                    queue!(out, SetForegroundColor(new.s.fg))?;
                    curr_fg = Some(new.s.fg);
                }
                if curr_bg != Some(new.s.bg) {
                    queue!(out, SetBackgroundColor(new.s.bg))?;
                    curr_bg = Some(new.s.bg);
                }
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
        out.flush()?;
        std::mem::swap(&mut self.cells, &mut old.cells);
        self.clear_buffer();
        Ok(())
    }
}
