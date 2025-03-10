'use strict';
;(function () {
    'use strict';

    let actionsDropdown = document.querySelector('.admin-list-page__actions-dropdown');
    let actionsDropdownCount = document.querySelector('.admin-list-page__actions-dropdown__count');
    let allCheckbox = document.querySelector('.admin-list-page__all-checkbox');
    let rowCheckboxes = document.querySelectorAll('.admin-list-page__row-checkbox');

    function changedRowCheckboxes() {
        let count = 0;
        for (let i = 0; i < rowCheckboxes.length; i++) {
            let rowCheckbox = rowCheckboxes[i];
            if (rowCheckbox.checked) count++;
        }

        if (actionsDropdownCount) actionsDropdownCount.innerHTML = String(count);

        if (count === 0) {
            if (actionsDropdown) {
                actionsDropdown.style.opacity = '0';
                actionsDropdown.style.pointerEvents = 'none';
            }
        } else {
            if (actionsDropdown) {
                actionsDropdown.style.opacity = '1';
                actionsDropdown.style.pointerEvents = 'auto';
            }
        }
    }

    function changedAllCheckbox(checked) {
        for (let i = 0; i < rowCheckboxes.length; i++) {
            let rowCheckbox = rowCheckboxes[i];
            rowCheckbox.checked = checked;
        }
    }

    function changedRowCheckbox() {
        if (allCheckbox) {
            let checked = true;
            for (let i = 0; i < rowCheckboxes.length; i++) {
                let rowCheckbox = rowCheckboxes[i];
                if (!rowCheckbox.checked) {
                    checked = false;
                    break;
                }
            }
            allCheckbox.checked = checked;
        }
    }

    function handleChangeAllCheckbox(e) {
        changedAllCheckbox(e.target.checked);
        changedRowCheckboxes();
    }

    function handleChangeRowCheckbox() {
        changedRowCheckbox();
        changedRowCheckboxes();
    }

    if (allCheckbox) {
        allCheckbox.removeEventListener('change', handleChangeAllCheckbox);
        allCheckbox.addEventListener('change', handleChangeAllCheckbox);
    }

    for (let i = 0; i < rowCheckboxes.length; i++) {
        let rowCheckbox = rowCheckboxes[i];

        rowCheckbox.removeEventListener('change', handleChangeRowCheckbox);
        rowCheckbox.addEventListener('change', handleChangeRowCheckbox);
    }

    if (allCheckbox) changedAllCheckbox(allCheckbox.checked);
    changedRowCheckbox();
    changedRowCheckboxes();
})();