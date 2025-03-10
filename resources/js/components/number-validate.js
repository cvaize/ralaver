'use strict';
;(function () {
    'use strict';

    let numElements = document.querySelectorAll('.js-number-validate');

    for (let i = 0; i < numElements.length; i++) {
        let numElement = numElements[i];
        numElement.addEventListener('change', validate);
        numElement.addEventListener('input', validate);
        numElement.addEventListener('keyup', validate);
        numElement.addEventListener('paste', validate);
        validate.call(numElement, true);
    }

    function validate(event, isSetCaret) {
        let caret = this.selectionStart;
        if (this.min) this.value = Math.max(parseInt(this.min), parseInt(String(this.value).replace(/\D/g, "")) || 0);
        if (this.max) this.value = Math.min(parseInt(this.max), parseInt(String(this.value).replace(/\D/g, "")) || 0);
        if (this.value) this.value = Number(this.value).toLocaleString();
        if (!isSetCaret) {
            this.setSelectionRange(caret, caret);
            this.selectionStart = caret;
        }
    }
})();