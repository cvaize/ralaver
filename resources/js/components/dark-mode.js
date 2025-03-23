'use strict';
;(function () {
    'use strict';
    let radioDark = document.getElementById('admin-dark-mode__radio--dark');
    let radioLight = document.getElementById('admin-dark-mode__radio--light');
    let radioAuto = document.getElementById('admin-dark-mode__radio--auto');

    if (radioDark && radioLight && radioAuto) {
        radioDark.addEventListener('change', toggleTheme)
        radioLight.addEventListener('change', toggleTheme)
        radioAuto.addEventListener('change', toggleTheme)
        toggleTheme();
    }

    function toggleTheme() {
        let mode = null;
        if (radioDark.checked) mode = 'dark';
        if (radioLight.checked) mode = 'light';
        if (radioAuto.checked) mode = 'auto';

        if (mode) {
            document.cookie = 'dark_mode=' + mode + '; path=/'
        }
    }
})();