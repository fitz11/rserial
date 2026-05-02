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
    y_locked: Option<(f64, f64)>,
}

impl<'a> GraphView<'a> {
    pub fn new(float_data: &'a [f64], int_data: &'a [i64], y_locked: Option<(f64, f64)>) -> Self {
        Self {
            float_data,
            int_data,
            y_locked,
        }
    }
}

impl Widget for GraphView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

        render_float_graph(self.float_data, self.y_locked, top, buf);
        render_int_graph(self.int_data, bottom, buf);
    }
}

fn render_float_graph(data: &[f64], y_locked: Option<(f64, f64)>, area: Rect, buf: &mut Buffer) {
    let color = DEFAULT_THEME.graph_float_border;

    if data.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(" Float (no data) ");
        block.render(area, buf);
        return;
    }

    let last = *data.last().unwrap();
    let data_min = data.iter().copied().fold(f64::INFINITY, f64::min);
    let data_max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let lock_indicator = if y_locked.is_some() {
        "Y: locked"
    } else {
        "Y: auto"
    };
    let title = format!(
        " Float [last: {last:.2} | min: {data_min:.2} max: {data_max:.2}] [{lock_indicator}] "
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title);

    let points: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();

    let y_bounds = if let Some((lo, hi)) = y_locked {
        [lo, hi]
    } else if (data_max - data_min).abs() < f64::EPSILON {
        [data_min - 1.0, data_max + 1.0]
    } else {
        let range = data_max - data_min;
        let padding = range * 0.1;
        [data_min - padding, data_max + padding]
    };

    let datasets = vec![Dataset::default()
        .data(&points)
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))];

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(Axis::default().bounds([0.0, (data.len().saturating_sub(1)) as f64]))
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

    let last = *data.last().unwrap();
    let min = data.iter().copied().min().unwrap();
    let max = data.iter().copied().max().unwrap();

    let title = format!(" Integer [last: {last} | min: {min} max: {max}] ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title);

    let points: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let y_bounds = if min == max {
        [min as f64 - 1.0, max as f64 + 1.0]
    } else {
        let range = (max - min) as f64;
        let padding = range * 0.1;
        [min as f64 - padding, max as f64 + padding]
    };

    let datasets = vec![Dataset::default()
        .data(&points)
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))];

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(Axis::default().bounds([0.0, (data.len().saturating_sub(1)) as f64]))
        .y_axis(Axis::default().bounds(y_bounds));

    chart.render(area, buf);
}
