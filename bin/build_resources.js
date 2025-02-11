const fs = require('fs');
const zlib = require('zlib');
const minify = require('@node-minify/core');
const cleanCSS = require('@node-minify/clean-css');
const uglifyjs = require('@node-minify/uglify-js');

const styles = [
    './resources/libraries/normalize/normalize.css',
    './resources/libraries/fancybox/fancybox.css',
    './resources/components/layout/layout.css',

    './resources/components/accordion/accordion.css',
    './resources/components/alert/alert.css',
    './resources/components/b-checkbox/b-checkbox.css',
    './resources/components/b-radio/b-radio.css',
    './resources/components/b-tabs/b-tabs.css',
    './resources/components/breadcrumb/breadcrumb.css',
    './resources/components/btn/btn.css',
    './resources/components/c-checkbox/c-checkbox.css',
    './resources/components/c-radio/c-radio.css',
    './resources/components/checkbox/checkbox.css',
    './resources/components/collapse/collapse.css',
    './resources/components/color-checkbox/color-checkbox.css',
    './resources/components/d-block/d-block.css',
    './resources/components/d-flex/d-flex.css',
    './resources/components/d-inline-block/d-inline-block.css',
    './resources/components/d-none/d-none.css',
    './resources/components/dark-mode/dark-mode.css',
    './resources/components/dropdown/dropdown.css',
    './resources/components/field/field.css',
    './resources/components/input/input.css',
    './resources/components/layout/layout.css',
    './resources/components/list-page/list-page.css',
    './resources/components/login/login.css',
    './resources/components/menu/menu.css',
    './resources/components/modal/modal.css',
    './resources/components/pagination/pagination.css',
    './resources/components/radio/radio.css',
    './resources/components/s-collapse/s-collapse.css',
    './resources/components/search-group/search-group.css',
    './resources/components/sidebar/sidebar.css',
    './resources/components/table/table.css',
    './resources/components/tabs/tabs.css',
    './resources/components/tabs/tabs--menu-mod.css',
    './resources/components/tabs/tabs--menu-mod-lg.css',
    './resources/components/tabs/tabs--menu-mod-md.css',
    './resources/components/tabs/tabs--menu-mod-sm.css',
    './resources/components/tabs/tabs--menu-mod-xl.css',
    './resources/components/tabs/tabs--menu-mod-xs.css',
    './resources/components/tabs/tabs--menu-mod-xxl.css',
    './resources/components/tag/tag.css',
];

const scripts = [
    './resources/libraries/embla-carousel/embla-carousel.umd.js',
    './resources/libraries/embla-carousel/embla-carousel-class-names.umd.js',
    './resources/libraries/fancybox/fancybox.umd.js',
    './resources/components/number-validate/number-validate.js',
];

async function runStyles(){
    let content = '';

    for (const style of styles) {
        content += fs.readFileSync(style);
    }

    fs.writeFileSync('./resources/build/app.css', content);

    await minify({
        compressor: cleanCSS,
        input: './resources/build/app.css',
        output: './resources/build/app.min.css'
    });

    content = fs.readFileSync('./resources/build/app.min.css');

    content = zlib.gzipSync(content, {level: 9});

    fs.writeFileSync('./resources/build/app.min.css.gz', content);

}

async function runScripts(){
    let content = '';

    for (const script of scripts) {
        content += fs.readFileSync(script);
    }

    fs.writeFileSync('./resources/build/app.js', content);

    await minify({
        compressor: uglifyjs,
        input: './resources/build/app.js',
        output: './resources/build/app.min.js',
    });

    content = fs.readFileSync('./resources/build/app.min.js');

    content = zlib.gzipSync(content, {level: 9});

    fs.writeFileSync('./resources/build/app.min.js.gz', content);
}

async function run(){
    await runStyles();
    await runScripts();
}

run();
