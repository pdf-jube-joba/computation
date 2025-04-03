// test module for using SVG.js

// write rectangle on id="svg_test" element with SVG.js
export async function write_rect() {;
    const draw = SVG().addTo('#svg_test').size(300, 300);
    const rect = draw.rect(100, 100).attr({ fill: '#f06' });
    rect.move(50, 50);
    console.log('Rectangle drawn');
}
