use super::Rect;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::stdout;
pub const FG: Color = Color::White;
pub const BG: Color = Color::Rgb { r: 0, g: 0, b: 0 };
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: FG,
            bg: BG,
        }
    }
}
pub struct ScreenBuffer {
    pub cells: Vec<Cell>,
    pub w: u16,
    pub h: u16,
}

impl ScreenBuffer {
    pub fn new(w:u16, h:u16)->Self{
        Self { cells: vec![Cell::default(); (w*h) as usize], w, h}
    }
    fn clear_buffer(&mut self) {
        self.cells.fill(Cell::default())
    }
    pub fn fill_rect(&mut self, r: Rect, c: Color) {
        if r.w == 0 || r.h == 0 {
            return;
        }
        for x in r.x..=r.right() {
            for y in r.y..=r.bottom() {
                self.set_cell_xy(
                    r,
                    x,
                    y,
                    Cell {
                        c: ' ',
                        fg: c,
                        bg: c,
                    },
                );
            }
        }
    }
    pub fn clear_rect(&mut self, r: Rect) {
        self.fill_rect(r, BG);
    }
    pub fn set_string_xy(&mut self, r: Rect, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
        let cur = self;
        for (i, c) in s.chars().enumerate() {
            let xx = x.saturating_add(i as u16);
            cur.set_cell_xy(r, xx, y, Cell { c, fg, bg });
        }
    }
    pub fn set_cell_xy(&mut self, r: Rect, x: u16, y: u16, cell: Cell) {
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
        let mut c = self;
        let mut p = old;
        let mut out = stdout().lock();
        let mut current_fg = None;
        let mut current_bg = None;

        for y in 0..c.h {
            let mut x = 0;
            while x < c.w {
                let idx = (y * c.w + x) as usize;
                let old = p.cells[idx];
                let new = c.cells[idx];
                if new == old {
                    x += 1;
                    continue;
                }
                let start_x = x;
                let style_fg = new.fg;
                let style_bg = new.bg;
                let mut line = String::new();
                while x < c.w {
                    let idx = (y * c.w + x) as usize;
                    let old = p.cells[idx];
                    let new = c.cells[idx];
                    //stop if unchanged
                    if new == old {
                        break;
                    }
                    // stop if style changes
                    if new.fg != style_fg || new.bg != style_bg {
                        break;
                    }
                    line.push(new.c);
                    x += 1;
                }
                queue!(out, MoveTo(start_x, y))?;
                if current_fg != Some(style_fg) {
                    queue!(out, SetForegroundColor(style_fg))?;
                    current_fg = Some(style_fg);
                }
                if current_bg != Some(style_bg) {
                    queue!(out, SetBackgroundColor(style_bg))?;
                    current_bg = Some(style_bg);
                }
                queue!(out, Print(line))?;
            }
        }
        queue!(out, ResetColor)?;
        std::mem::swap(&mut c, &mut p);
        c.clear_buffer();
        Ok(())
    }
}
