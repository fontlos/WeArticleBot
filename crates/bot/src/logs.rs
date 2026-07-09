//! 日志

use logforth::append;
use logforth::layout::TextLayout;
use logforth::record::Level;
use logforth::record::LevelFilter;

pub fn init() {
    logforth::starter_log::builder()
        .dispatch(|d| {
            d.filter(LevelFilter::MoreSevereEqual(Level::Debug))
                .append(append::Stdout::default().with_layout(TextLayout::default()))
        })
        .apply();
}
