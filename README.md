# Puffin 
a small, TUI framework written in Rust with a retained UI system, mostly for personal use.

# Usage

```Rust
use crossterm::{event::KeyCode, style::Color};
use puffin::*;
struct MyWidget;
impl Widget<Ctx, Cmd> for MyWidget {
    fn draw(&mut self, r: Rect, screen: &mut ScreenBuffer, focused: bool) {
        r.fill(" ", Color::White, Color::Black, screen);
    }
    fn behaviour(&mut self, k: crossterm::event::KeyEvent, ctx: &mut Ctx) -> Response<Cmd> {
        match k.code {
            KeyCode::Char('q') => Response::Cmd(Cmd::Quit),
            _ => Response::Ignored,
        }
    }
}
struct Ctx;
enum Cmd {
    Quit,
}
fn main() -> std::io::Result<()> {
    let mut ctx = Ctx;
    puffin::new(Direction::Vertical, |root| {
        root.vertical(|inner| inner.add(MyWidget).add(MyWidget))
    })
    .run(&mut ctx, |key, res, ctx, ui| match res {
        Response::Ignored => match key.code {
            KeyCode::Char('h') => ui.focus(FocusDirection::Left),
            KeyCode::Char('j') => ui.focus(FocusDirection::Down),
            KeyCode::Char('k') => ui.focus(FocusDirection::Up),
            KeyCode::Char('l') => ui.focus(FocusDirection::Right),
            _ => {}
        },
        Response::Cmd(_) => ui.quit(),
        Response::Consumed => {}
    })
}
```
