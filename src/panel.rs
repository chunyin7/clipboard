use dispatch2::{MainThreadBound, run_on_main};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, DefinedClass, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSColor, NSControlTextEditingDelegate, NSFont,
    NSMainMenuWindowLevel, NSPanel, NSScrollView, NSTableCellView, NSTableColumn, NSTableView,
    NSTableViewDataSource, NSTableViewDelegate, NSTextField, NSView, NSWindowCollectionBehavior,
    NSWindowStyleMask, NSAutoresizingMaskOptions, NSLineBreakMode, NSUserInterfaceItemIdentification,
};
use objc2_foundation::{MainThreadMarker, NSInteger, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};
use objc2_foundation::ns_string;
use std::cell::RefCell;
use std::convert::TryFrom;

use crate::models::{ClipboardEntry, History};

const CONTENT_TAG: NSInteger = 1;
const TIMESTAMP_TAG: NSInteger = 2;
const ROW_HEIGHT: f64 = 44.0;
const HORIZONTAL_INSET: f64 = 10.0;
const TOP_INSET: f64 = 4.0;
const LABEL_SPACING: f64 = 2.0;
const CONTENT_FONT_SIZE: f64 = 14.0;
const TIMESTAMP_FONT_SIZE: f64 = 12.0;
const CONTENT_HEIGHT: f64 = 20.0;
const TIMESTAMP_HEIGHT: f64 = 14.0;
const PANEL_WIDTH: f64 = 320.0;
const PANEL_HEIGHT: f64 = 300.0;
const PANEL_CORNER_RADIUS: f64 = 12.0;

fn cell_identifier() -> &'static NSString {
    ns_string!("HistoryCell")
}

#[derive(Default)]
struct HistoryDataSourceState {
    rows: RefCell<Vec<ClipboardEntry>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "HistoryDataSource"]
    #[ivars = HistoryDataSourceState]
    struct HistoryDataSource;

    impl HistoryDataSource {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(HistoryDataSourceState::default());
            unsafe { objc2::msg_send![super(this), init] }
        }

        #[unsafe(method(numberOfRowsInTableView:))]
        fn number_of_rows(&self, _table_view: &NSTableView) -> NSInteger {
            self.row_count_as_nsinteger()
        }

        #[unsafe(method_id(tableView:objectValueForTableColumn:row:))]
        fn object_value(
            &self,
            _table_view: &NSTableView,
            _column: Option<&NSTableColumn>,
            row: NSInteger,
        ) -> Option<Retained<AnyObject>> {
            self.string_for_row(row)
                .map(|string| string.into_super().into_super())
        }
    }
);

unsafe impl NSObjectProtocol for HistoryDataSource {}
unsafe impl NSTableViewDataSource for HistoryDataSource {}
unsafe impl NSControlTextEditingDelegate for HistoryDataSource {}

unsafe impl NSTableViewDelegate for HistoryDataSource {
    fn tableView_viewForTableColumn_row(
        &self,
        table_view: &NSTableView,
        table_column: Option<&NSTableColumn>,
        row: NSInteger,
    ) -> Option<Retained<NSView>> {
        let _ = table_column;
        let entry = self.entry_for_row(row)?;
        let width = table_view.bounds().size.width;
        let mtm = table_view.mtm();

        let cell: Retained<NSTableCellView> = unsafe {
            table_view
                .makeViewWithIdentifier_owner(cell_identifier(), None)
                .map(|view| Retained::cast_unchecked(view))
        }
        .unwrap_or_else(|| self.build_cell(mtm, width));

        self.update_cell(&cell, &entry, width);

        Some(cell.into_super())
    }
}

impl HistoryDataSource {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { objc2::msg_send![Self::alloc(mtm), init] }
    }

    pub fn with_history(mtm: MainThreadMarker, history: &History) -> Retained<Self> {
        let data_source = Self::new(mtm);
        data_source.reload_from_history(history);
        data_source
    }

    pub fn reload_from_history(&self, history: &History) {
        let snapshot = {
            let locked = history.lock().expect("history mutex poisoned");
            locked.clone()
        };

        let ivars = self.ivars();
        *ivars.rows.borrow_mut() = snapshot;
    }

    fn row_count(&self) -> usize {
        let ivars = self.ivars();
        ivars.rows.borrow().len()
    }

    fn row_count_as_nsinteger(&self) -> NSInteger {
        self.row_count() as NSInteger
    }

    fn string_for_row(&self, row: NSInteger) -> Option<Retained<NSString>> {
        self.entry_for_row(row)
            .map(|entry| NSString::from_str(entry.content.as_str()))
    }

    fn entry_for_row(&self, row: NSInteger) -> Option<ClipboardEntry> {
        let index = usize::try_from(row).ok()?;
        let ivars = self.ivars();
        ivars.rows.borrow().get(index).cloned()
    }

    fn timestamp_text(entry: &ClipboardEntry) -> String {
        entry.timestamp
            .format("%b %-d, %Y at %-I:%M %p")
            .to_string()
    }

    fn update_cell(&self, cell: &NSTableCellView, entry: &ClipboardEntry, width: f64) {
        if let Some(content_label) = unsafe { cell.textField() } {
            let content_text = NSString::from_str(entry.content.as_str());
            content_label.setStringValue(&content_text);
            self.layout_label(&content_label, width, true);
        }

        if let Some(timestamp_view) = cell.viewWithTag(TIMESTAMP_TAG) {
            let timestamp_label: Retained<NSTextField> =
                unsafe { Retained::cast_unchecked(timestamp_view) };
            let timestamp_text = NSString::from_str(Self::timestamp_text(entry).as_str());
            timestamp_label.setStringValue(&timestamp_text);
            self.layout_label(&timestamp_label, width, false);
        }
    }

    fn layout_label(&self, label: &NSTextField, width: f64, is_content: bool) {
        let available_width = (width - (HORIZONTAL_INSET * 2.0)).max(0.0);
        let (height, y) = if is_content {
            (
                CONTENT_HEIGHT,
                TOP_INSET + TIMESTAMP_HEIGHT + LABEL_SPACING,
            )
        } else {
            (
                TIMESTAMP_HEIGHT,
                TOP_INSET,
            )
        };

        label.setFrame(NSRect::new(
            NSPoint::new(HORIZONTAL_INSET, y),
            NSSize::new(available_width, height),
        ));
    }

    fn build_cell(
        &self,
        mtm: MainThreadMarker,
        width: f64,
    ) -> Retained<NSTableCellView> {
        let cell_frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, ROW_HEIGHT));
        let cell = NSTableCellView::initWithFrame(NSTableCellView::alloc(mtm), cell_frame);
        cell.setIdentifier(Some(cell_identifier()));

        let content_label = NSTextField::labelWithString(ns_string!(""), mtm);
        content_label.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);
        content_label.setMaximumNumberOfLines(1);
        content_label.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
        let content_font = NSFont::systemFontOfSize(CONTENT_FONT_SIZE);
        content_label.setFont(Some(&content_font));
        content_label.setTag(CONTENT_TAG);
        self.layout_label(&content_label, width, true);
        unsafe { cell.setTextField(Some(&content_label)); }
        cell.addSubview(&content_label);

        let timestamp_label = NSTextField::labelWithString(ns_string!(""), mtm);
        timestamp_label.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);
        timestamp_label.setMaximumNumberOfLines(1);
        timestamp_label.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
        let timestamp_font = NSFont::systemFontOfSize(TIMESTAMP_FONT_SIZE);
        timestamp_label.setFont(Some(&timestamp_font));
        let secondary_color = NSColor::secondaryLabelColor();
        timestamp_label.setTextColor(Some(&secondary_color));
        timestamp_label.setTag(TIMESTAMP_TAG);
        self.layout_label(&timestamp_label, width, false);
        cell.addSubview(&timestamp_label);

        cell
    }
}

pub struct PanelController {
    panel: Retained<NSPanel>,
    table_view: Retained<NSTableView>,
    data_source: Retained<HistoryDataSource>,
    history: History,
}

impl PanelController {
    pub fn new(mtm: MainThreadMarker, history: History) -> Self {
        let app = NSApplication::sharedApplication(mtm);

        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(PANEL_WIDTH, PANEL_HEIGHT));
        let style = NSWindowStyleMask::HUDWindow
            | NSWindowStyleMask::NonactivatingPanel
            | NSWindowStyleMask::UtilityWindow;

        let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
            NSPanel::alloc(mtm),
            frame,
            style,
            NSBackingStoreType::Buffered,
            false,
        );

        panel.setLevel(NSMainMenuWindowLevel + 1);
        panel.setOpaque(false);
        panel.setHasShadow(true);
        panel.setHidesOnDeactivate(true);
        panel.setCollectionBehavior(NSWindowCollectionBehavior::CanJoinAllSpaces);
        panel.center();

        let content_view = NSView::initWithFrame(NSView::alloc(mtm), frame);
        content_view.setWantsLayer(true);
        if let Some(layer) = content_view.layer() {
            layer.setCornerRadius(PANEL_CORNER_RADIUS);
            layer.setMasksToBounds(true);
        }

        let data_source = HistoryDataSource::with_history(mtm, &history);
        let table_view = NSTableView::initWithFrame(NSTableView::alloc(mtm), frame);

        let column_identifier = NSString::from_str("content");
        let column_title = NSString::from_str("Content");
        let column =
            NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), &column_identifier);
        column.setTitle(&column_title);
        table_view.addTableColumn(&column);

        unsafe {
            let data_source_proto: &ProtocolObject<dyn NSTableViewDataSource> =
                ProtocolObject::from_ref(&*data_source);
            table_view.setDataSource(Some(data_source_proto));

            let delegate_proto: &ProtocolObject<dyn NSTableViewDelegate> =
                ProtocolObject::from_ref(&*data_source);
            table_view.setDelegate(Some(delegate_proto));
        }
        table_view.setRowHeight(ROW_HEIGHT);
        table_view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        table_view.setHeaderView(None);
        table_view.reloadData();

        let scroll_view = NSScrollView::initWithFrame(NSScrollView::alloc(mtm), frame);
        scroll_view.setHasVerticalScroller(true);
        scroll_view.setAutohidesScrollers(true);
        scroll_view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        scroll_view.setDocumentView(Some(&table_view));

        content_view.addSubview(&scroll_view);
        panel.setContentView(Some(&content_view));
        app.activate();

        Self {
            panel,
            table_view,
            data_source,
            history,
        }
    }

    pub fn show(&self) {
        let app = NSApplication::sharedApplication(self.marker());
        #[allow(deprecated)]
        app.activateIgnoringOtherApps(true);
        self.panel.orderFrontRegardless();
    }

    pub fn hide(&self) {
        self.panel.orderOut(None)
    }

    pub fn is_visible(&self) -> bool {
        self.panel.isVisible()
    }

    pub fn toggle(&self) {
        if self.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }

    pub fn refresh_history(&self) {
        self.data_source.reload_from_history(&self.history);
        self.table_view.reloadData();
    }

    fn marker(&self) -> MainThreadMarker {
        self.panel.mtm()
    }
}

pub fn init_panel(history: History) -> MainThreadBound<PanelController> {
    run_on_main(move |mtm| {
        let controller = PanelController::new(mtm, history.clone());
        let marker = controller.marker();
        MainThreadBound::new(controller, marker)
    })
}
