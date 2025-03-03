use tinytemplate::TinyTemplate;

pub fn new() -> TinyTemplate<'static> {
    let mut tt = TinyTemplate::new();

    tt.add_template("pages.error.default", RESOURCES_PAGES_ERROR_DEFAULT_HTML).unwrap();
    tt.add_template("pages.home.index", RESOURCES_PAGES_HOME_INDEX_HTML).unwrap();
    tt.add_template("pages.home.user", RESOURCES_PAGES_HOME_USER_HTML).unwrap();
    tt.add_template("pages.auth.login", RESOURCES_PAGES_AUTH_LOGIN_HTML).unwrap();
    tt.add_template("pages.auth.register", RESOURCES_PAGES_AUTH_REGISTER_HTML).unwrap();
    tt.add_template("pages.auth.forgot-password", RESOURCES_PAGES_AUTH_FORGOT_PASSWORD_HTML).unwrap();
    tt.add_template("pages.auth.forgot-password-confirm", RESOURCES_PAGES_AUTH_FORGOT_PASSWORD_CONFIRM_HTML).unwrap();
    tt
}

static RESOURCES_PAGES_ERROR_DEFAULT_HTML: &str = include_str!("../../resources/pages/error/default.html");
static RESOURCES_PAGES_HOME_INDEX_HTML: &str = include_str!("../../resources/pages/home/index.html");
static RESOURCES_PAGES_HOME_USER_HTML: &str = include_str!("../../resources/pages/home/user.html");
static RESOURCES_PAGES_AUTH_LOGIN_HTML: &str = include_str!("../../resources/pages/auth/login.html");
static RESOURCES_PAGES_AUTH_REGISTER_HTML: &str = include_str!("../../resources/pages/auth/register.html");
static RESOURCES_PAGES_AUTH_FORGOT_PASSWORD_HTML: &str = include_str!("../../resources/pages/auth/forgot-password.html");
static RESOURCES_PAGES_AUTH_FORGOT_PASSWORD_CONFIRM_HTML: &str = include_str!("../../resources/pages/auth/forgot-password-confirm.html");