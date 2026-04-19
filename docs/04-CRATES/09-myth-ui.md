# `myth-ui` — TUI 대시보드

## 역할

myth의 **관찰용 TUI**. `myth status`, `myth watch`로 호출되며 caselog, lesson, Migration Readiness, 실행 중인 task를 실시간 보여준다.

**Claude Code 세션과 독립**. 별도 터미널에서 띄우는 뷰어. 사용자가 "지금 myth에 뭐가 일어나고 있지?"를 한눈에 볼 수 있게.

**의존**: `myth-common`, `myth-db`.
**의존받음**: `myth-cli`.

## Cargo.toml

```toml
[package]
name = "myth-ui"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
myth-common = { path = "../myth-common" }
myth-db = { path = "../myth-db" }

ratatui = { workspace = true }
crossterm = { workspace = true }
syntect = { workspace = true }
pulldown-cmark = { workspace = true }

serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
tokio = { workspace = true, features = ["time", "sync", "fs"] }
tracing = { workspace = true }
anyhow = { workspace = true }
```

## 모듈 구조

```
crates/myth-ui/
└── src/
    ├── lib.rs              # UI 공개 API
    ├── app.rs              # App 상태 머신
    ├── events.rs           # 키 입력, 파일 watch, tick
    ├── layout.rs           # ratatui 레이아웃 계산
    ├── panels/
    │   ├── mod.rs
    │   ├── caselog.rs      # 최근 caselog 스트리밍
    │   ├── lessons.rs      # 활성 lesson 목록
    │   ├── brief.rs        # brief.md 프리뷰
    │   ├── migration.rs    # Migration Readiness A~E
    │   └── tasks.rs        # 실행 중 task (orchestrator 통합)
    ├── markdown.rs         # pulldown-cmark + 터미널 스타일
    ├── syntax.rs           # syntect 구문 강조
    └── theme.rs            # 색상·스타일 정의
```

## 레이아웃

```
┌─── myth ─────────────────────────────────────────────────────┐
│ Session: abc...  |  Active Tasks: 2  |  [q]uit [h]elp       │
├──────────────────┬───────────────────────────────────────────┤
│ Caselog (latest) │ Brief                                     │
│                  │                                           │
│ 14:23 L3 Bash    │ # Weekly Brief (2026-W16)                │
│ 14:22 L2 Edit    │                                           │
│ 14:20 L4 Bash    │ 활성 lesson 23개                          │
│ 14:18 L1 Read    │ 신규 lesson 2개                           │
│ ...              │ Lapsed 1개                                │
│                  │ ...                                       │
├──────────────────┤                                           │
│ Active Tasks     │                                           │
│                  │ ## Migration Readiness                    │
│ [1] T1.1 running │                                           │
│ [2] T1.2 done    │ A [·] Assessor Tier review (3w)          │
│ [3] T1.3 waiting │ B [x] Vector store migration             │
│                  │ C [ ] Gavel daemon (P99 6.7ms)           │
├──────────────────┤ D [ ] Semantic detection                  │
│ Lessons          │ E [ ] AST validation                      │
│                  │                                           │
│ L3-0012  MEDIUM  │                                           │
│ L4-0015  HIGH    │                                           │
│ L2-0023  LOW     │                                           │
│ ...              │                                           │
└──────────────────┴───────────────────────────────────────────┘
```

**3분할 레이아웃**:
- 좌상: Caselog 스트리밍
- 좌중: Active Tasks (orchestrator)
- 좌하: 활성 Lesson 목록
- 우: Brief + Migration Readiness (마크다운 렌더)

## `App` 상태 머신

```rust
pub struct App {
    pub mode: AppMode,
    pub should_quit: bool,
    
    // 패널 상태
    pub caselog: CaselogPanel,
    pub lessons: LessonsPanel,
    pub brief: BriefPanel,
    pub migration: MigrationPanel,
    pub tasks: TasksPanel,
    
    // 포커스
    pub focused_panel: PanelId,
}

pub enum AppMode {
    Normal,
    Help,
    LessonDetail(LessonId),
}

pub enum PanelId {
    Caselog,
    Lessons,
    Brief,
    Migration,
    Tasks,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            mode: AppMode::Normal,
            should_quit: false,
            caselog: CaselogPanel::new()?,
            lessons: LessonsPanel::new()?,
            brief: BriefPanel::new()?,
            migration: MigrationPanel::new()?,
            tasks: TasksPanel::new()?,
            focused_panel: PanelId::Caselog,
        })
    }
    
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => self.handle_key(key)?,
            Event::Tick => self.refresh().await?,
            Event::FileChanged(path) => self.handle_file_change(path)?,
        }
        Ok(())
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.mode == AppMode::Help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                self.mode = AppMode::Normal;
            }
            return Ok(());
        }
        
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('h') | KeyCode::Char('?') => self.mode = AppMode::Help,
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::Char('j') | KeyCode::Down => self.focused_panel_mut().scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.focused_panel_mut().scroll_up(),
            KeyCode::Enter => self.activate_selection()?,
            _ => {}
        }
        Ok(())
    }
}
```

## `events.rs` — 이벤트 수집

```rust
pub enum Event {
    Tick,
    Key(KeyEvent),
    FileChanged(PathBuf),
}

pub async fn event_stream() -> impl Stream<Item = Event> {
    let (tx, rx) = mpsc::channel(100);
    
    // Tick (200ms마다)
    let tx_tick = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(200));
        loop {
            interval.tick().await;
            if tx_tick.send(Event::Tick).await.is_err() { break; }
        }
    });
    
    // Key input
    let tx_key = tx.clone();
    tokio::spawn(async move {
        loop {
            if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
                if let Ok(crossterm::event::Event::Key(k)) = crossterm::event::read() {
                    if tx_key.send(Event::Key(k)).await.is_err() { break; }
                }
            }
        }
    });
    
    // 파일 watch (notify crate)
    let tx_file = tx.clone();
    tokio::spawn(async move {
        // caselog.jsonl, brief.md 변경 감지
        // 실제로는 notify::RecommendedWatcher 사용
    });
    
    ReceiverStream::new(rx)
}
```

## `panels/caselog.rs`

```rust
pub struct CaselogPanel {
    entries: VecDeque<CaselogEntry>,
    scroll_offset: usize,
    max_entries: usize,  // 기본 100
}

impl CaselogPanel {
    pub fn new() -> Result<Self> {
        let mut panel = Self {
            entries: VecDeque::new(),
            scroll_offset: 0,
            max_entries: 100,
        };
        panel.load_recent()?;
        Ok(panel)
    }
    
    pub fn load_recent(&mut self) -> Result<()> {
        let path = myth_common::caselog_path();
        let reader = JsonlReader::new(&path);
        
        // tail 100개
        let recent: Vec<CaselogEntry> = reader.tail(100)?;
        self.entries.clear();
        self.entries.extend(recent);
        Ok(())
    }
    
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let block = Block::default()
            .title(" Caselog ")
            .borders(Borders::ALL)
            .border_style(border_style);
        
        let items: Vec<ListItem> = self.entries.iter()
            .skip(self.scroll_offset)
            .take(area.height as usize - 2)
            .map(|entry| {
                let level_color = match entry.level {
                    Level::Info => Color::Gray,
                    Level::Low => Color::Blue,
                    Level::Medium => Color::Yellow,
                    Level::High => Color::LightRed,
                    Level::Critical => Color::Red,
                };
                let ts = entry.ts.format("%H:%M");
                
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", ts), Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("L{} ", entry.level as u8),
                        Style::default().fg(level_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&entry.tool),
                    Span::styled(
                        format!(" {}", truncate(&entry.summary, 40)),
                        Style::default().fg(Color::White),
                    ),
                ]))
            })
            .collect();
        
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
```

## `panels/migration.rs` — Migration Readiness

```rust
pub struct MigrationPanel {
    milestones: Vec<MilestoneStatus>,
}

pub struct MilestoneStatus {
    pub id: char,               // A, B, C, D, E
    pub title: String,
    pub triggered: bool,
    pub current_value: String,  // 예: "P99: 6.7ms"
    pub threshold: String,      // "15ms"
    pub notes: Vec<String>,
}

impl MigrationPanel {
    pub fn refresh(&mut self) -> Result<()> {
        self.milestones = vec![
            self.compute_milestone_a()?,  // Assessor Tier review
            self.compute_milestone_b()?,  // Vector store
            self.compute_milestone_c()?,  // Gavel daemon (hook-latency.ndjson 집계)
            self.compute_milestone_d()?,  // Semantic detection
            self.compute_milestone_e()?,  // AST validation
        ];
        Ok(())
    }
    
    fn compute_milestone_c(&self) -> Result<MilestoneStatus> {
        // hook-latency.ndjson 최근 14일 P99 계산
        let latencies = load_hook_latencies_last_14d()?;
        let p99 = compute_percentile(&latencies, 0.99);
        
        let triggered = p99 > 15.0;
        
        Ok(MilestoneStatus {
            id: 'C',
            title: "Gavel daemon migration".into(),
            triggered,
            current_value: format!("P99: {:.1}ms", p99),
            threshold: "15ms".into(),
            notes: vec![
                format!("Duration: 14 days"),
                format!("Sample: {} events", latencies.len()),
            ],
        })
    }
}
```

## `markdown.rs` — 마크다운 렌더링

brief.md를 터미널에서 보기 좋게:

```rust
pub fn render_markdown(markdown: &str, width: u16) -> Text<'_> {
    use pulldown_cmark::{Parser, Event, Tag};
    
    let parser = Parser::new(markdown);
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut style = Style::default();
    
    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, _, _)) => {
                style = Style::default()
                    .fg(match level {
                        pulldown_cmark::HeadingLevel::H1 => Color::Cyan,
                        pulldown_cmark::HeadingLevel::H2 => Color::Yellow,
                        _ => Color::White,
                    })
                    .add_modifier(Modifier::BOLD);
            }
            Event::End(_) => {
                if !current_line.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut current_line)));
                }
                style = Style::default();
            }
            Event::Text(text) => {
                current_line.push(Span::styled(text.to_string(), style));
            }
            Event::SoftBreak => current_line.push(Span::raw(" ")),
            Event::HardBreak => {
                lines.push(Line::from(std::mem::take(&mut current_line)));
            }
            _ => {}
        }
    }
    
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }
    
    Text::from(lines)
}
```

## `syntax.rs` — 코드 구문 강조

```rust
pub fn highlight_code(code: &str, lang: &str) -> Vec<Line> {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    
    let syntax = ss.find_syntax_by_token(lang)
        .unwrap_or(ss.find_syntax_plain_text());
    let theme = &ts.themes["base16-ocean.dark"];
    
    let mut highlighter = HighlightLines::new(syntax, theme);
    
    code.lines().map(|line| {
        let ranges = highlighter.highlight_line(line, &ss).unwrap();
        let spans: Vec<Span> = ranges.iter()
            .map(|(style, text)| {
                Span::styled(
                    text.to_string(),
                    Style::default().fg(convert_color(style.foreground)),
                )
            })
            .collect();
        Line::from(spans)
    }).collect()
}
```

## 진입점

```rust
pub async fn run_dashboard() -> Result<()> {
    // terminal 초기화
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new()?;
    let mut events = event_stream().await;
    
    while !app.should_quit {
        terminal.draw(|f| layout::render(&app, f))?;
        
        if let Some(event) = events.next().await {
            app.handle_event(event).await?;
        }
    }
    
    // 정리
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    
    Ok(())
}
```

## 키 바인딩 (vim 스타일)

```
q, Esc     종료
h, ?       도움말
Tab        다음 패널로 포커스 이동
Shift+Tab  이전 패널
j, ↓       아래로
k, ↑       위로
Enter      선택
r          수동 새로고침
g          맨 처음으로
G          맨 끝으로
/          검색 (간단한 텍스트 매칭)
```

## 테스트

```
tests/
├── markdown_render_test.rs    # pulldown-cmark 출력 검증
├── syntax_highlight_test.rs
├── panel_layout_test.rs       # rectangle 계산
└── event_handling_test.rs     # 키 매핑
```

인터랙티브 TUI는 `crossterm` 이벤트 mock으로 테스트.

## 관련 결정

- Decision 8 (Ultraplan 문서 분할): TUI는 Day-1에 구현되나 최소 기능 우선
- 카테고리 7 (brief.md): 이 패널에서 렌더링
- Decision 7 (Migration Readiness): 이 패널에서 시각화
