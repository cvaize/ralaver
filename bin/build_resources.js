const fs = require('fs');
const zlib = require('zlib');
const minify = require('@node-minify/core');
const cleanCSS = require('@node-minify/clean-css');
const uglifyjs = require('@node-minify/uglify-js');

const styles = [
    ...fs.readdirSync('./resources/css/libraries')
        .filter(s => s.endsWith('.css'))
        .map(s => './resources/css/libraries/' + s),

    './resources/css/components/layout.css',

    ...fs.readdirSync('./resources/css/components')
        .filter(s => s.endsWith('.css'))
        .map(s => './resources/css/components/' + s),
].filter((item, i, ar) => ar.indexOf(item) === i);

const scripts = [
    ...fs.readdirSync('./resources/js/libraries')
        .filter(s => s.endsWith('.js'))
        .map(s => './resources/js/libraries/' + s),

    ...fs.readdirSync('./resources/js/components')
        .filter(s => s.endsWith('.js'))
        .map(s => './resources/js/components/' + s),
].filter((item, i, ar) => ar.indexOf(item) === i);

const svg = [
    ...fs.readdirSync('./resources/svg')
        .filter(s => s.endsWith('.svg'))
        .map(s => './resources/svg/' + s),
].filter((item, i, ar) => ar.indexOf(item) === i);

async function runStyles(){
    let content = '';

    for (const style of styles) {
        content += fs.readFileSync(style);
    }

    fs.writeFileSync('./resources/dist/app.css', content);

    await minify({
        compressor: cleanCSS,
        input: './resources/dist/app.css',
        output: './resources/dist/app.min.css'
    });

    content = fs.readFileSync('./resources/dist/app.min.css');

    content = zlib.gzipSync(content, {level: 9});

    fs.writeFileSync('./resources/dist/app.min.css.gz', content);

}

async function runScripts(){
    let content = '';

    for (const script of scripts) {
        content += fs.readFileSync(script);
    }

    fs.writeFileSync('./resources/dist/app.js', content);

    await minify({
        compressor: uglifyjs,
        input: './resources/dist/app.js',
        output: './resources/dist/app.min.js',
    });

    content = fs.readFileSync('./resources/dist/app.min.js');

    content = zlib.gzipSync(content, {level: 9});

    fs.writeFileSync('./resources/dist/app.min.js.gz', content);
}
async function runSvg(){
    for (const svgElement of svg) {
        let content = fs.readFileSync(svgElement);
        let path = svgElement.replace('./resources/svg/', './resources/dist/').trim();
        fs.writeFileSync(path, content);

        content = zlib.gzipSync(content, {level: 9});

        fs.writeFileSync(`${path}.gz`, content);
    }

}

async function run(){
    await runStyles();
    await runScripts();
    await runSvg();
}

run();
