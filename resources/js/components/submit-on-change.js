'use strict';
;(function () {
    'use strict';
    let elements
        = document.querySelectorAll('.js-submit-on-change');

    for (let i = 0; i < elements.length; i++) {
        elements[i].addEventListener('change', handleChange)
    }

    function handleChange(e) {
        if (e.target && e.target.form) {
            e.target.form.submit();
        }
    }
})();