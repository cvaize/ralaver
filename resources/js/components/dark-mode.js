'use strict';
;(function () {
    'use strict';

    let darkMode = document.querySelector('#admin-dark-mode__checkbox');
    let action, method;

    if (darkMode) {
        action = darkMode.getAttribute("data-action");
        method = darkMode.getAttribute("data-method");
        if (action && method) {
            darkMode.removeEventListener("change", onChange);
            darkMode.addEventListener("change", onChange);
        }
    }

    function onChange() {
        fetch(action, {
            method,
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({dark_mode: darkMode.checked})
        })
    }
})();