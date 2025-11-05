use dispatch2::{MainThreadBound, run_on_main};
use objc2::rc::Retained;
use objc2::{ClassType, MainThreadOnly, define_class};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSMainMenuWindowLevel, NSPanel, NSTableColumn, NSTableView,
    NSView, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSInteger, NSObject, NSPoint, NSRect, NSSize, NSString};

use crate::models::History;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "HistoryDataSource"]
    struct HistoryDataSource;

    impl HistoryDataSource {
        #[unsafe(method(numberOfRowsInTableView))]
        fn number_of_rows(&self, _table_view: &NSTableView) -> NSInteger {
            // TODO: return the row count once rows are populated
            todo!("implement number_of_rows")
        }

        #[unsafe(method_id(tableView:objectValueForTableColumn:row:))]
        fn object_value(
            &self,
            _table_view: &NSTableView,
            _column: Option<&NSTableColumn>,
            _row: NSInteger,
        ) -> Option<Retained<NSString>> {
            // TODO: look up the entry in the rows array and return an NSString
            todo!("implement object_value")
        }
    }
);

pub struct PanelController {
    panel: Retained<NSPanel>,
    history: History,
}

impl PanelController {
    pub fn new(mtm: MainThreadMarker, history: History) -> Self {
        let app = unsafe { NSApplication::sharedApplication(mtm) };

        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(360.0, 420.0));
        let style = NSWindowStyleMask::HUDWindow
            | NSWindowStyleMask::NonactivatingPanel
            | NSWindowStyleMask::UtilityWindow;

        let panel = unsafe {
            NSPanel::initWithContentRect_styleMask_backing_defer(
                NSPanel::alloc(mtm),
                frame,
                style,
                NSBackingStoreType::Buffered,
                false,
            )
        };

        unsafe {
            panel.setLevel(NSMainMenuWindowLevel + 1);
            panel.setOpaque(false);
            panel.setHasShadow(true);
            panel.setHidesOnDeactivate(true);
            panel.setCollectionBehavior(NSWindowCollectionBehavior::CanJoinAllSpaces);
        }

        let content_view = unsafe { NSView::initWithFrame(NSView::alloc(mtm), frame) };
        unsafe {
            panel.setContentView(Some(&content_view));
            app.activate();
        }

        Self { panel, history }
    }

    pub fn show(&self) {
        unsafe { self.panel.makeKeyAndOrderFront(None) }
    }

    pub fn hide(&self) {
        unsafe { self.panel.orderOut(None) }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { self.panel.isVisible() }
    }

    pub fn toggle(&self) {
        if self.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }

    pub fn refresh_history(&self) {
        let _ = &self.history;
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
