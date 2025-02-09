use tinytemplate::TinyTemplate;

pub fn new() -> TinyTemplate<'static> {
    let mut tt = TinyTemplate::new();

    tt.add_template("pages.error.default", RESOURCES_VIEW_PAGES_ERROR_DEFAULT_HTML).unwrap();
    tt.add_template("pages.home.index", RESOURCES_VIEW_PAGES_HOME_INDEX_HTML).unwrap();
    tt.add_template("pages.home.user", RESOURCES_VIEW_PAGES_HOME_USER_HTML).unwrap();
    tt
}

static RESOURCES_VIEW_PAGES_ERROR_DEFAULT_HTML: &str = include_str!("../../resources/view/pages/error/default.html");
static RESOURCES_VIEW_PAGES_HOME_INDEX_HTML: &str = include_str!("../../resources/view/pages/home/index.html");
static RESOURCES_VIEW_PAGES_HOME_USER_HTML: &str = include_str!("../../resources/view/pages/home/user.html");