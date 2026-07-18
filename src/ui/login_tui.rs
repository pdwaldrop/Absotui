use crate::login_app::AppLogin;
use crate::login_app::AppViewLogin;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};


/// init widget for selected `AppView` 
impl Widget for &mut AppLogin {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.view_state {
            AppViewLogin::Auth => self.render_auth(area, buf),
        }
    }
}


/// Rendering logic
impl AppLogin {

    fn render_auth(&mut self, _area: Rect, _buf: &mut Buffer) {

        let _ = self.auth();


    }

}

