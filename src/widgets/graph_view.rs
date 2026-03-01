use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    symbols::Marker,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Widget},
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
}

impl Widget for GraphView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

        render_float_graph(self.float_data, top, buf);
        render_int_graph(self.int_data, bottom, buf);
    }
}

fn render_float_graph(data: &[f64], area: Rect, buf: &mut Buffer) {
    let color = DEFAULT_THEME.graph_float_border;

    if data.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(" Float (no data) ");
        block.render(area, buf);
        return;
    }

    // Calculate how many points fit in the inner width (area minus borders)
    let inner_width = area.width.saturating_sub(2) as usize;
    let visible = if data.len() > inner_width {
        &data[data.len() - inner_width..]
    } else {
        data
    };

    let last = *visible.last().unwrap();
    let min = visible.iter().copied().fold(f64::INFINITY, f64::min);
    let max = visible.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let title = format!(" Float [last: {last:.2} | min: {min:.2} max: {max:.2}] ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title);

    let points: Vec<(f64, f64)> = visible
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();

    let y_bounds = if (max - min).abs() < f64::EPSILON {
        [min - 1.0, max + 1.0]
    } else {
        [min, max]
    };

    let datasets = vec![Dataset::default()
        .data(&points)
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))];

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(Axis::default().bounds([0.0, (visible.len().saturating_sub(1)) as f64]))
        .y_axis(Axis::default().bounds(y_bounds));

    chart.render(area, buf);
}

fn render_int_graph(data: &[i64], area: Rect, buf: &mut Buffer) {
    let color = DEFAULT_THEME.graph_int_border;

    if data.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(" Integer (no data) ");
        block.render(area, buf);
        return;
    }

    let inner_width = area.width.saturating_sub(2) as usize;
    let visible = if data.len() > inner_width {
        &data[data.len() - inner_width..]
    } else {
        data
    };

    let last = *visible.last().unwrap();
    let min = visible.iter().copied().min().unwrap();
    let max = visible.iter().copied().max().unwrap();

    let title = format!(" Integer [last: {last} | min: {min} max: {max}] ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title);

    let points: Vec<(f64, f64)> = visible
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let y_bounds = if min == max {
        [min as f64 - 1.0, max as f64 + 1.0]
    } else {
        [min as f64, max as f64]
    };

    let datasets = vec![Dataset::default()
        .data(&points)
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))];

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(Axis::default().bounds([0.0, (visible.len().saturating_sub(1)) as f64]))
        .y_axis(Axis::default().bounds(y_bounds));

    chart.render(area, buf);
}
