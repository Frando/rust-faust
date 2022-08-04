function sigLog(n) {
    return Math.log(Math.abs(n) + 1) / Math.log(10) * sig(n);
}

function sig(n) {
    return n == 0 ? 0 : Math.abs(n) / n;
}

function sigExp(n) {
    return (Math.pow(10, Math.abs(n)) - 1) * sig(n);
}

function value2px(value, valueMin, valueMax, pxMin, pxMax) {
    var valueWidth = sigLog(valueMax) - sigLog(valueMin);
    var pixelWidth = pxMax - pxMin;
    var ratio = pixelWidth / valueWidth;

    return ratio * (sigLog(value) - sigLog(valueMin)) + pxMin;
}

function px2value(px, valueMin, valueMax, pxMin, pxMax) {
    var valueWidth = sigLog(valueMax) - sigLog(valueMin);
    console.log('valueWidth', valueWidth)
    var pixelWidth = pxMax - pxMin;
    var ratio = pixelWidth / valueWidth;

    return sigExp((px - pxMin) / ratio + sigLog(valueMin));
}

function prettify(n) {
    var exp = Math.round(Math.pow(10, Math.log(Math.abs(n)) / Math.log(10)));
    return exp == 0 ? 0 : Math.round(n / exp) * exp;
}

var valueMin = -70,
    valueMax = 0,
    pxMin = 10,
    pxMax = 210;

var parts = 9;
for (var i = 0; i <= parts; i++) {
    var px = pxMin + (pxMax - pxMin) / parts * i;
    var exactValue = px2value(px, valueMin, valueMax, pxMin, pxMax);
    var prettyValue = prettify(exactValue);
    console.log(prettyValue, valueMin, valueMax, px);
}
