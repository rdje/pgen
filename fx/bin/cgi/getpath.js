
function span_onmouseover (span) {
 document.bg = span.style.background;
 document.fg = span.style.color;

 span.style.background = "YellowGreen";
 span.style.color      = "Brown";
}

function span_onmouseout (span) {
 span.style.background = document.bg;
 span.style.color      = document.fg;
}
