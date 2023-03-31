use tui::{Frame, backend::CrosstermBackend, layout::{Constraint, Layout, Direction, Rect, Alignment}, text::{Spans, Span}, style::{Style, Modifier, Color}, widgets::{Paragraph, Block, Borders, Wrap, List, ListItem, ListState}};

use crate::app::{App, UiWidget};
use tui::backend::Backend;

type RenderFn<B> = fn (&mut Frame<B>, App, Rect);


pub fn render_home<B: Backend>(f: &mut Frame<B>, app: App) {
    let widgets = app.active_widgets();
    let mut pre_main: Vec<(Constraint, RenderFn<B>)> = vec![];
    let mut post_main: Vec<(Constraint, RenderFn<B>)> = vec![];
    // let mut constraints = [Constraint::Min(2)];

    for widget in widgets {
        match widget {
            UiWidget::Search => pre_main.push((Constraint::Length(3), render_search::<B>)),
            UiWidget::Debug => post_main.push((Constraint::Length(2 + App::DEBUG_SIZE), render_debug::<B>))
        }
    }

    let mut widgets: Vec<(Constraint, RenderFn<B>)> = vec![];
    widgets.append(&mut pre_main);
    widgets.push((Constraint::Min(2), render_main::<B>));
    widgets.append(&mut post_main);

    let constraints: Vec<Constraint> = widgets.iter().map(|(c, _)| c.clone()).collect();

    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    for (i, (chunk, (_, render))) in chunks.iter().zip(widgets).enumerate() {
        render(f, app.clone(), chunks[i]);
    }
}

fn render_debug<B: Backend>(f: &mut Frame<B>, app: App, layout_chunk: Rect) {
    let mut text = vec![];
    for debug in app.get_debug() {
        text.push(Spans::from(Span::styled(debug, Style::default())));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Debug").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, layout_chunk);

}

fn render_search<B: Backend>(f: &mut Frame<B>, app: App, layout_chunk: Rect) {
    let text = vec![
        Spans::from(vec![
            Span::raw(""),
            Span::styled("🔎 ", Style::default()),
            Span::styled("Not implemented yet 😅", Style::default().fg(Color::LightYellow)),
        ]),
    ];


    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, layout_chunk);
}

fn render_main<B: Backend>(f: &mut Frame<B>, app: App, layout_chunk: Rect) {
    // TODO: Render the view based on the active route
    //       The route should be stored in the app
    //       e.g. app.route = Route::RingOverview, app.route = Route::NodeDetail
    render_ring_overwiew(f, app, layout_chunk);
}


fn render_ring_overwiew<B: Backend>(f: &mut Frame<B>, app: App, layout_chunk: Rect) {
    let text = vec![
        Spans::from(Span::styled("Hello", Style::default().fg(Color::Red))),
        Spans::from(Span::styled("World", Style::default().fg(Color::Blue))),
    ];

    let (list_state, ids) = app.node_ids();
    let items: Vec<ListItem> = ids.iter().map(|id| ListItem::new(id.to_string())).collect();
    let list = List::new(items)
        .block(Block::default().title("Nodes").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Rgb(255, 204, 153)).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    let mut list_state = list_state;
    

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Main").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(50),
            Constraint::Min(50),
        ])
        .split(layout_chunk);

    f.render_stateful_widget(list, chunks[0], &mut list_state);
    f.render_widget(paragraph, chunks[1]);
}
