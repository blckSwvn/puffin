use super::Rect;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{
        Attribute, Attributes, Color, Print, ResetColor, SetAttribute, SetAttributes, SetBackgroundColor, SetForegroundColor
    },
};
use std::{collections::HashMap, io::stdout, sync::Arc};
pub const FG: Color = Color::White;
pub const BG: Color = Color::Rgb { r: 0, g: 0, b: 0 };
#[derive(Clone, Copy, PartialEq, Eq)]
struct ContKind(u32);
impl ContKind{
    const EMPTY: Self = Self(0);
    const CONTINUE: Self = Self(1);
    const TAG_MASK: u32 = 0b11;
    const SHIFT: u32 = 2;
    const MAX_INDEX: u32 = (1 << 30)-1;
    fn grapheme(i:u32)->Self{
        debug_assert!(i <= Self::MAX_INDEX);
        Self((i << Self::SHIFT) | 0b10)
    }
    fn grapheme_index(self)->Option<u32>{
        if self.0 & Self::TAG_MASK == 0b10{
            Some(self.0 >> 2)
        }else{
            None
        }
    }
    fn is_empty(self)->bool{
        self == Self::EMPTY
    }
    fn is_continue(self)->bool{
        self == Self::CONTINUE
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Cell {
    c: ContKind,
    s: Colors,
    a: Attributes
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Colors{
    fg:Color,
    bg:Color,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ContKind::EMPTY,
            s: Colors { fg: FG, bg: BG },
            a: Attributes::none()
        }
    }
}
impl Cell{
    fn grapheme(i: u32, fg:Color, bg:Color, a:Attributes)->Self{
        Self{
            c:ContKind::grapheme(i),
            s:Colors { fg, bg },
            a,
        }
    }
    fn continuation(fg:Color, bg:Color)->Self{
        Self{
            c:ContKind::CONTINUE,
            s:Colors { fg, bg },
            a:Attributes::none()
        }
    }
}
use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;
pub struct Screen{
    curr:Vec<Cell>,
    old:Vec<Cell>,
    graphemes:Vec<Arc<str>>,
    lookup:HashMap<Arc<str>, u32>,
    w:u16,
    h:u16
}

impl Screen{
    pub fn new(w:u16, h:u16)->Self{
        Self {
            curr: vec![Cell::default(); (w*h) as usize],
            old:  vec![Cell::default(); (w*h) as usize],
            graphemes: vec![],
            lookup: HashMap::new(),
            w, h
        }
    }
    fn clear(&mut self){
        self.curr.fill(Cell::default());
    }
    fn intern(&mut self, s:&str)->u32{
        if let Some(&i) = self.lookup.get(s){
            return i;
        }
        let i = self.graphemes.len() as u32;
        debug_assert!(i <= ContKind::MAX_INDEX);
        let val:Arc<str> = Arc::from(s);
        self.graphemes.push(val.clone());
        self.lookup.insert(val, i);
        i
    }
pub fn set(
    &mut self,
    r: Rect,
    x: u16,
    y: u16,
    s: &str,
    fg: Color,
    bg: Color,
    a: Attributes,
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
        let index = self.intern(grapheme);
        let idx = y as usize * self.w as usize + cx as usize;
        self.curr[idx] = Cell::grapheme(index, fg, bg, a);
        // Mark continuation cells for wide graphemes.
        for offset in 1..width {
            let idx = y as usize * self.w as usize
                + cx as usize
                + offset;
            self.curr[idx] = Cell::continuation(fg, bg);
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
            if self.curr[idx].c.is_continue() {
                start_x -= 1;
            } else {
                break;
            }
        }
        let idx = row_start + start_x as usize;
        let Some(grapheme_index) = self.curr[idx].c.grapheme_index() else {
            return;
        };
        let Some(grapheme) = self.graphemes.get(grapheme_index as usize) else {
            return;
        };
        let width = grapheme.width();

        for offset in 0..width {
            let idx = row_start + start_x as usize + offset;
            self.curr[idx] = Cell::default();
        }
    }
    pub fn draw(&mut self) -> std::io::Result<()> {
        use std::io::Write;
        let mut out = stdout().lock();
        let (mut curr_fg, mut curr_bg, mut curr_attr) = (None, None, None);
        for y in 0..self.h {
            let mut x = 0;
            while x < self.w {
                let idx = y as usize * self.w as usize + x as usize;
                let new = &self.curr[idx];
                let prev = &self.old[idx];
                if new == prev {
                    x += 1;
                    continue;
                }
                if new.c.is_continue(){
                    x += 1;
                    continue;
                }
                queue!(out, MoveTo(x, y))?;
                if curr_attr != Some(new.a) {
                    queue!(out, SetAttribute(Attribute::Reset))?;
                    if !new.a.is_empty() {
                        queue!(out, SetAttributes(new.a))?;
                    }
                    curr_attr = Some(new.a);
                    curr_fg = None;
                    curr_bg = None;
                }
                if curr_fg != Some(new.s.fg) {
                    queue!(out, SetForegroundColor(new.s.fg))?;
                    curr_fg = Some(new.s.fg);
                }
                if curr_bg != Some(new.s.bg) {
                    queue!(out, SetBackgroundColor(new.s.bg))?;
                    curr_bg = Some(new.s.bg);
                }
                if let Some(index) = new.c.grapheme_index() {
                    let grapheme = &self.graphemes[index as usize];
                    queue!(out, Print(grapheme))?;
                    x += grapheme.width() as u16;
                } else {
                    queue!(out, Print(" "))?;
                    x += 1;
                }
            }
        }
        queue!(out, ResetColor, SetAttribute(Attribute::Reset))?;
        out.flush()?;
        std::mem::swap(&mut self.curr, &mut self.old);
        self.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn print_memory_sizes() {
        println!("ContKind: {} bytes", std::mem::size_of::<ContKind>());
        println!("Cell: {} bytes", std::mem::size_of::<Cell>());
        println!("Colors: {} bytes", std::mem::size_of::<Colors>());
        println!("Attributes: {} bytes", std::mem::size_of::<Attributes>());
        println!("Color: {} bytes", std::mem::size_of::<Color>());
        println!("Box<str>: {} bytes", std::mem::size_of::<Box<str>>());

        let (w, h) = (188u16, 48u16);
        let cells = w as usize * h as usize;

        println!("\nFor a {w}x{h} terminal ({cells} cells):");
        println!("One buffer: {} bytes", cells * std::mem::size_of::<Cell>());
        println!("Two buffers: {} bytes", cells * 2 * std::mem::size_of::<Cell>());
    }
}
