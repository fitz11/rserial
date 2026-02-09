use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Sparkline, Widget},
};

use crate::constants::DEFAULT_THEME;

pub struct GraphView<'a> {
    float_data: &'a [f64],
    int_data: &'a [i64],
}

impl<'a> GraphView<'a> {
    pub fn new(float_data: &'a [f64], int_data: &'a [i64]) -> Self {
        Self {
            float_data,
            int_data,
        }
    }

    fn scale_floats(data: &[f64]) -> Vec<u64> {
        if data.is_empty() {
            return Vec::new();
        }

        let min = data.iter().copied().fold(f64::INFINITY, f64::min);
        let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        if range == 0.0 {
            return vec![500; data.len()];
        }

        data.iter()
            .map(|&v| ((v - min) / range * 1000.0) as u64)
            .collect()
    }

    fn scale_ints(data: &[i64]) -> Vec<u64> {
        if data.is_empty() {
            return Vec::new();
        }

        let min = data.iter().copied().min().unwrap();
        let max = data.iter().copied().max().unwrap();
        let range = max - min;

        if range == 0 {
            return vec![500; data.len()];
        }

        data.iter()
            .map(|&v| ((v - min) as f64 / range as f64 * 1000.0) as u64)
            .collect()
    }
}

impl Widget for GraphView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

        // Float graph
        let float_title = if self.float_data.is_empty() {
            " Float (no data) ".to_string()
        } else {
            let last = self.float_data.last().unwrap();
            let min = self
                .float_data
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
            let max = self
                .float_data
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            format!(" Float [last: {last:.2} | min: {min:.2} max: {max:.2}] ")
        };
        let float_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DEFAULT_THEME.graph_float_border))
            .title(float_title);
        let scaled_floats = Self::scale_floats(self.float_data);
        Sparkline::default()
            .block(float_block)
            .data(&scaled_floats)
            .max(1000)
            .style(Style::default().fg(DEFAULT_THEME.graph_float_border))
            .render(top, buf);

        // Integer graph
        let int_title = if self.int_data.is_empty() {
            " Integer (no data) ".to_string()
        } else {
            let last = self.int_data.last().unwrap();
            let min = self.int_data.iter().copied().min().unwrap();
            let max = self.int_data.iter().copied().max().unwrap();
            format!(" Integer [last: {last} | min: {min} max: {max}] ")
        };
        let int_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DEFAULT_THEME.graph_int_border))
            .title(int_title);
        let scaled_ints = Self::scale_ints(self.int_data);
        Sparkline::default()
            .block(int_block)
            .data(&scaled_ints)
            .max(1000)
            .style(Style::default().fg(DEFAULT_THEME.graph_int_border))
            .render(bottom, buf);
    }
}
