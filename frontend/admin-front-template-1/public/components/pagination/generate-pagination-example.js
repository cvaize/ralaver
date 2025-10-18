;(function () {
    'use strict';

    let boxes = document.querySelectorAll('.js-generate-pagination-example');

    for (let i = 0; i < boxes.length; i++) {
        let box = boxes[i];

        function generate(page, totalPages, offset) {
            let result = [];

            let resultLength = 5 + offset * 2;

            if (resultLength >= totalPages) {
                for (let j = 1; j <= totalPages; j++) {
                    result.push(j);
                }

                return result;
            }

            result.push(1);

            let start = page - offset;
            let end = page + offset;
            let isStartDot = true;
            let isEndDot = true;

            if (start <= 3) {
                start = 2;
                end = 3 + offset * 2;
                isStartDot = false;
            }

            if (end >= totalPages - 2) {
                if (isStartDot) {
                    start = totalPages - (2 + offset * 2);
                }

                end = totalPages - 1;
                isEndDot = false;
            }

            if (start <= 3) {
                start = 2;
                isStartDot = false;
            }

            if (isStartDot) {
                result.push(0);
            }

            for (let j = start; j <= end; j++) {
                result.push(j);
            }

            if (isEndDot) {
                result.push(0);
            }

            result.push(totalPages);

            return result;
        }

        function build(array, currentPage) {
            let result = '';

            for (let i = 0; i < array.length; i++) {
                let item = array[i];
                let active = currentPage === item ? ' admin-active' : '';
                if (item === 0) {
                    result += `<li class="admin-pagination__item"><span class="admin-pagination__link">...</span></li>`;
                } else {
                    result += `<li class="admin-pagination__item"><a class="admin-pagination__link${active}" href="#">${item}</a></li>`;
                }

            }

            return `<nav aria-label="Page navigation example" style="margin-bottom: 1rem;">
                    <ul class="admin-pagination">
                        ${result}
                    </ul>
                </nav>`;
        }

        let html = '';

        function apply(page, totalPages, offset, testArray) {
            let array = generate(page, totalPages, offset);
            console.assert(array.toString() === testArray.toString(), `[${array.toString()}] not equal [${testArray.toString()}] - page=${page}`);
            return build(array, page);
        }

        html += apply(-11, 5, 1, [1, 2, 3, 4, 5]);
        html += apply(1, 5, 1, [1, 2, 3, 4, 5]);
        html += apply(2, 5, 1, [1, 2, 3, 4, 5]);
        html += apply(3, 5, 1, [1, 2, 3, 4, 5]);
        html += apply(4, 5, 1, [1, 2, 3, 4, 5]);
        html += apply(5, 5, 1, [1, 2, 3, 4, 5]);
        html += apply(11, 5, 1, [1, 2, 3, 4, 5]);

        html += apply(-111, 10, 1, [1, 2, 3, 4, 5, 0, 10]);
        html += apply(1, 10, 1, [1, 2, 3, 4, 5, 0, 10]);
        html += apply(2, 10, 1, [1, 2, 3, 4, 5, 0, 10]);
        html += apply(3, 10, 1, [1, 2, 3, 4, 5, 0, 10]);
        html += apply(4, 10, 1, [1, 2, 3, 4, 5, 0, 10]);
        html += apply(5, 10, 1, [1, 0, 4, 5, 6, 0, 10]);
        html += apply(6, 10, 1, [1, 0, 5, 6, 7, 0, 10]);
        html += apply(7, 10, 1, [1, 0, 6, 7, 8, 9, 10]);
        html += apply(8, 10, 1, [1, 0, 6, 7, 8, 9, 10]);
        html += apply(9, 10, 1, [1, 0, 6, 7, 8, 9, 10]);
        html += apply(10, 10, 1, [1, 0, 6, 7, 8, 9, 10]);
        html += apply(111, 10, 1, [1, 0, 6, 7, 8, 9, 10]);

        html += apply(-222, 20, 2, [1, 2, 3, 4, 5, 6, 7, 0, 20]);
        html += apply(1, 20, 2, [1, 2, 3, 4, 5, 6, 7, 0, 20]);
        html += apply(2, 20, 2, [1, 2, 3, 4, 5, 6, 7, 0, 20]);
        html += apply(3, 20, 2, [1, 2, 3, 4, 5, 6, 7, 0, 20]);
        html += apply(4, 20, 2, [1, 2, 3, 4, 5, 6, 7, 0, 20]);
        html += apply(5, 20, 2, [1, 2, 3, 4, 5, 6, 7, 0, 20]);
        html += apply(6, 20, 2, [1, 0, 4, 5, 6, 7, 8, 0, 20]);
        html += apply(7, 20, 2, [1, 0, 5, 6, 7, 8, 9, 0, 20]);
        html += apply(8, 20, 2, [1, 0, 6, 7, 8, 9, 10, 0, 20]);
        html += apply(9, 20, 2, [1, 0, 7, 8, 9, 10, 11, 0, 20]);
        html += apply(10, 20, 2, [1, 0, 8, 9, 10, 11, 12, 0, 20]);
        html += apply(11, 20, 2, [1, 0, 9, 10, 11, 12, 13, 0, 20]);
        html += apply(12, 20, 2, [1, 0, 10, 11, 12, 13, 14, 0, 20]);
        html += apply(13, 20, 2, [1, 0, 11, 12, 13, 14, 15, 0, 20]);
        html += apply(14, 20, 2, [1, 0, 12, 13, 14, 15, 16, 0, 20]);
        html += apply(15, 20, 2, [1, 0, 13, 14, 15, 16, 17, 0, 20]);
        html += apply(16, 20, 2, [1, 0, 14, 15, 16, 17, 18, 19, 20]);
        html += apply(17, 20, 2, [1, 0, 14, 15, 16, 17, 18, 19, 20]);
        html += apply(18, 20, 2, [1, 0, 14, 15, 16, 17, 18, 19, 20]);
        html += apply(19, 20, 2, [1, 0, 14, 15, 16, 17, 18, 19, 20]);
        html += apply(20, 20, 2, [1, 0, 14, 15, 16, 17, 18, 19, 20]);
        html += apply(222, 20, 2, [1, 0, 14, 15, 16, 17, 18, 19, 20]);

        html += build(generate(1, 100, 100), 1);

        box.innerHTML = html;
    }
})();